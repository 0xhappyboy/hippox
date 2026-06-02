//! Plan-and-Execute mode workflow execution

use crate::executors::SkillCall;
use crate::skill_scheduler::SkillScheduler;
use crate::t;
use serde_json::Value;
use std::collections::HashMap;
use tracing::info;

use super::core::WorkflowExecutor;
use super::prompt;
use super::types::*;
use super::utils::VARIABLE_REGEX;

/// Parse plan response from LLM
pub fn parse_plan_response(response: &str) -> anyhow::Result<PlanInstruction> {
    let json_str = WorkflowExecutor::extract_json(response);
    let instruction: PlanInstruction = serde_json::from_str(&json_str)?;
    Ok(instruction)
}

/// Resolve value reference
fn resolve_value_ref(value_ref: &ValueRef, context: &Workflow) -> Value {
    match value_ref {
        ValueRef::Literal(value) => value.clone(),
        ValueRef::Reference(ref_reference) => {
            let path = &ref_reference.path;
            if let Some(value) = context.get_variable(path) {
                value.clone()
            } else if path == "user_input" {
                Value::Null
            } else {
                Value::Null
            }
        }
        ValueRef::Expression(expr) => Value::String(expr.expr.clone()),
    }
}

/// Resolve variables deep with cached regex
fn resolve_variables_deep(value: &Value, context: &HashMap<String, Value>) -> Value {
    if let Some(s) = value.as_str() {
        if s.contains("{{") && s.contains("}}") {
            let mut result = s.to_string();
            for cap in VARIABLE_REGEX.captures_iter(s) {
                let var_name = &cap[1];
                if let Some(val) = context.get(var_name) {
                    let replacement = if let Some(num) = val.as_f64() {
                        num.to_string()
                    } else if let Some(s) = val.as_str() {
                        s.to_string()
                    } else {
                        val.to_string()
                    };
                    result = result.replace(&format!("{{{{{}}}}}", var_name), &replacement);
                }
            }
            return Value::String(result);
        }
        return Value::String(s.to_string());
    }
    if let Some(s) = value.as_str() {
        if s.starts_with("{{") && s.ends_with("}}") {
            let var_name = &s[2..s.len() - 2];
            if let Some(val) = context.get(var_name) {
                return val.clone();
            }
            return Value::String(s.to_string());
        }
        return Value::String(s.to_string());
    }
    if let Some(obj) = value.as_object() {
        let mut new_obj = serde_json::Map::new();
        for (k, v) in obj {
            new_obj.insert(k.clone(), resolve_variables_deep(v, context));
        }
        return Value::Object(new_obj);
    }
    if let Some(arr) = value.as_array() {
        let new_arr: Vec<Value> = arr
            .iter()
            .map(|v| resolve_variables_deep(v, context))
            .collect();
        return Value::Array(new_arr);
    }
    value.clone()
}

/// Evaluate condition
fn evaluate_condition(condition: &Condition, context: &Workflow) -> bool {
    let left = resolve_value_ref(&condition.left, context);
    let right = resolve_value_ref(&condition.right, context);
    match condition.op.as_str() {
        "eq" => left == right,
        "ne" => left != right,
        "gt" => {
            if let (Some(left_num), Some(right_num)) = (left.as_u64(), right.as_u64()) {
                left_num > right_num
            } else if let (Some(left_num), Some(right_num)) = (left.as_f64(), right.as_f64()) {
                left_num > right_num
            } else {
                false
            }
        }
        "lt" => {
            if let (Some(left_num), Some(right_num)) = (left.as_u64(), right.as_u64()) {
                left_num < right_num
            } else if let (Some(left_num), Some(right_num)) = (left.as_f64(), right.as_f64()) {
                left_num < right_num
            } else {
                false
            }
        }
        "contains" => left
            .as_str()
            .map(|s| s.contains(right.as_str().unwrap_or("")))
            .unwrap_or(false),
        _ => false,
    }
}

/// Execute step with retry
async fn execute_step_with_retry(
    executor: &WorkflowExecutor,
    skill_name: &str,
    parameters: HashMap<String, Value>,
    step: &WorkflowStep,
) -> anyhow::Result<String> {
    let max_retries = step
        .on_error
        .as_ref()
        .and_then(|e| e.max_retries)
        .unwrap_or(1);
    let mut last_error = None;
    for attempt in 0..max_retries {
        let call = SkillCall {
            action: skill_name.to_string(),
            parameters: parameters.clone(),
        };
        match executor.get_executor().execute(&call).await {
            Ok(output) => return Ok(output),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries - 1 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        (100 * (attempt + 1)).into(),
                    ))
                    .await;
                }
            }
        }
    }
    Err(anyhow::anyhow!(
        last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error"))
    ))
}

/// Execute workflow plan
async fn execute_workflow_plan(
    executor: &WorkflowExecutor,
    plan: &WorkflowPlan,
) -> anyhow::Result<String> {
    let mut context = Workflow::new();
    for (key, value) in &plan.parameters {
        context.set_variable(key, value.clone());
    }
    let mut string_context = HashMap::new();
    for step in &plan.steps {
        if let Some(condition) = &step.condition {
            if !evaluate_condition(condition, &context) {
                info!("Step {} condition not met, skipping", step.id);
                continue;
            }
        }
        let mut resolved_params = HashMap::new();
        for (key, value_ref) in &step.parameters {
            let resolved = resolve_value_ref(value_ref, &context);
            let final_resolved = resolve_variables_deep(&resolved, &string_context);
            resolved_params.insert(key.clone(), final_resolved);
        }
        let result = execute_step_with_retry(executor, &step.action, resolved_params, step).await;
        match result {
            Ok(output) => {
                if let Some(output_as) = &step.output_as {
                    context.set_variable(output_as, Value::String(output.clone()));
                    string_context.insert(output_as.clone(), Value::String(output.clone()));
                }
                context.add_step_result(WorkflowStepResult {
                    step_id: step.id.clone(),
                    skill: step.action.clone(),
                    input: step
                        .parameters
                        .iter()
                        .map(|(k, v)| (k.clone(), value_ref_to_value(v)))
                        .collect(),
                    output: output.clone(),
                    success: true,
                    error: None,
                });
            }
            Err(e) => {
                if let Some(error_handler) = &step.on_error {
                    match error_handler.action.as_str() {
                        "skip" => {
                            context.add_step_result(WorkflowStepResult {
                                step_id: step.id.clone(),
                                skill: step.action.clone(),
                                input: step
                                    .parameters
                                    .iter()
                                    .map(|(k, v)| (k.clone(), value_ref_to_value(v)))
                                    .collect(),
                                output: String::new(),
                                success: false,
                                error: Some(e.to_string()),
                            });
                            continue;
                        }
                        "fail" => {
                            return Err(anyhow::anyhow!("Step {} failed: {}", step.id, e));
                        }
                        _ => {
                            return Err(anyhow::anyhow!("Step {} failed: {}", step.id, e));
                        }
                    }
                } else {
                    return Err(anyhow::anyhow!("Step {} failed: {}", step.id, e));
                }
            }
        }
    }
    if let Some(last_result) = context.get_step_results().last() {
        Ok(last_result.output.clone())
    } else {
        Ok(t!("skill.no_steps_executed").to_string())
    }
}

/// Value ref to value
fn value_ref_to_value(value_ref: &ValueRef) -> Value {
    match value_ref {
        ValueRef::Literal(value) => value.clone(),
        ValueRef::Reference(ref_reference) => Value::String(format!("$ref:{}", ref_reference.path)),
        ValueRef::Expression(expr) => Value::String(format!("$expr:{}", expr.expr)),
    }
}

/// Execute plan-and-execute mode workflow
pub async fn execute_plan_and_execute(
    executor: &WorkflowExecutor,
    scheduler: &SkillScheduler,
    input: &str,
    skills_registry: &str,
    instances_registry: &str,
) -> String {
    let plan_prompt = prompt::build_plan_prompt(skills_registry, instances_registry, input);
    let llm_response = match scheduler.get_llm().generate(&plan_prompt).await {
        Ok(resp) => resp,
        Err(e) => return format!("{}: {}", t!("error.llm_error"), e),
    };
    let instruction = match parse_plan_response(&llm_response) {
        Ok(instr) => instr,
        Err(e) => return format!("Failed to parse plan: {}", e),
    };
    let task_id = executor.get_task_id().map(|s| s.to_string());
    match instruction {
        PlanInstruction {
            mode,
            plan,
            message,
        } => {
            if mode == "done" {
                return message.unwrap_or_else(|| t!("skill.no_actions_executed").to_string());
            }

            if let Some(plan) = plan {
                match execute_workflow_plan(executor, &plan).await {
                    Ok(result) => result,
                    Err(e) => {
                        if let Some(cb) = executor.get_callback() {
                            if let Some(ref tid) = task_id {
                                cb.on_workflow_failed(tid, &e.to_string()).await;
                            }
                        }
                        format!("Workflow failed: {}", e)
                    }
                }
            } else {
                t!("skill.no_actions_executed").to_string()
            }
        }
    }
}
