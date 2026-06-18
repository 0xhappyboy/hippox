//! Plan-and-Execute mode workflow execution

use crate::{SkillScheduler, TASK_STEP_SIGNAL_BUS};
use crate::prompts::build_plan_prompt;
use crate::t;
use hippox_atomic_skills::{SkillCall, SkillCallback, SkillContext};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

use super::core::WorkflowExecutor;
use super::types::*;
use super::utils::VARIABLE_REGEX;

pub fn parse_plan_response(response: &str) -> anyhow::Result<PlanInstruction> {
    let json_str = WorkflowExecutor::extract_json(response);
    let instruction: PlanInstruction = serde_json::from_str(&json_str)?;
    Ok(instruction)
}

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

async fn check_step_interruption(
    task_id: Option<&str>,
    callback: &Option<Arc<dyn WorkflowCallback>>,
    step_id: &str,
    step_name: &str,
    step_index: usize,
    checkpoint: Option<String>,
) -> Result<(), WorkflowExecutionResult> {
    if let Some(tid) = task_id {
        if let Some(state_updater) = crate::tasks::get_state_updater(tid).await {
            if state_updater.is_cancelled().await {
                if let Some(cb) = callback {
                    let info = StepInterruptionInfo {
                        interrupted: true,
                        reason: "cancelled".to_string(),
                        step_index,
                        step_name: step_name.to_string(),
                        checkpoint: checkpoint.clone(),
                    };
                    cb.on_step_interrupted(tid, &info).await;
                    cb.on_workflow_cancelled(tid, 0, step_index).await;
                }
                return Err(WorkflowExecutionResult::Cancelled {
                    completed_steps: step_index,
                });
            }
            if state_updater.is_paused().await {
                if let Some(ref checkpoint_data) = checkpoint {
                    state_updater.save_checkpoint(checkpoint_data).await;
                }
                if let Some(cb) = callback {
                    let info = StepInterruptionInfo {
                        interrupted: true,
                        reason: "paused".to_string(),
                        step_index,
                        step_name: step_name.to_string(),
                        checkpoint: checkpoint.clone(),
                    };
                    cb.on_step_interrupted(tid, &info).await;
                    cb.on_workflow_paused(tid, checkpoint.as_deref(), 0, step_index)
                        .await;
                }
                return Err(WorkflowExecutionResult::Paused {
                    checkpoint: checkpoint.clone(),
                    completed_steps: step_index,
                    partial_output: String::new(),
                });
            }
        }
    }
    Ok(())
}

async fn execute_step_with_retry(
    executor: &WorkflowExecutor,
    skill_name: &str,
    parameters: HashMap<String, Value>,
    step: &WorkflowStep,
    step_index: usize,
    task_id: Option<String>,
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
        let skill_callback_arc: Option<Arc<dyn SkillCallback>> = executor.get_skill_callback();
        let skill_context = SkillContext {
            task_id: task_id.clone(),
            skill_index: Some(step_index),
            skill_name: Some(skill_name.to_string()),
            extra: HashMap::new(),
            signal_bus: Some(&TASK_STEP_SIGNAL_BUS),
        };
        match executor
            .get_executor()
            .execute(&call, skill_callback_arc.as_deref(), Some(&skill_context))
            .await
        {
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

async fn execute_workflow_plan(
    executor: &WorkflowExecutor,
    plan: &WorkflowPlan,
    task_id: Option<&str>,
) -> anyhow::Result<(String, usize, usize)> {
    let mut context = Workflow::new();
    for (key, value) in &plan.parameters {
        context.set_variable(key, value.clone());
    }
    let mut string_context = HashMap::new();
    let mut success_count = 0;
    let mut failed_count = 0;
    for (idx, step) in plan.steps.iter().enumerate() {
        let checkpoint = serde_json::to_string(&WorkflowCheckpoint {
            last_completed_step: idx,
            variables: context.variables.clone(),
            completed_results: vec![],
            mode: WorkflowMode::PlanAndExecute,
            metadata: HashMap::new(),
        })
        .ok();
        match check_step_interruption(
            task_id,
            executor.get_workflow_callback(),
            &step.id,
            &step.action,
            idx,
            checkpoint.clone(),
        )
        .await
        {
            Ok(_) => {}
            Err(result) => {
                return Err(anyhow::anyhow!("{:?}", result));
            }
        }
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
        let result = execute_step_with_retry(
            executor,
            &step.action,
            resolved_params,
            step,
            idx,
            task_id.map(|s| s.to_string()),
        )
        .await;
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
                success_count += 1;
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
                            failed_count += 1;
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

    let final_output = if let Some(last_result) = context.get_step_results().last() {
        last_result.output.clone()
    } else {
        t!("skill.no_steps_executed").to_string()
    };

    Ok((final_output, success_count, failed_count))
}

fn value_ref_to_value(value_ref: &ValueRef) -> Value {
    match value_ref {
        ValueRef::Literal(value) => value.clone(),
        ValueRef::Reference(ref_reference) => Value::String(format!("$ref:{}", ref_reference.path)),
        ValueRef::Expression(expr) => Value::String(format!("$expr:{}", expr.expr)),
    }
}

pub async fn execute_plan_and_execute_with_categories(
    executor: &WorkflowExecutor,
    scheduler: &SkillScheduler,
    input: &str,
    categories: &[String],
) -> WorkflowExecutionResult {
    let overall_start = Instant::now();
    let task_id = executor.get_task_id().map(|s| s.to_string());
    let filtered_skills = crate::prompts::generate_skills_registry_by_categories(categories);
    let plan_prompt = crate::prompts::build_plan_prompt_with_categories(&filtered_skills, input);
    let llm_response = match scheduler
        .generate_with_task(&plan_prompt, &task_id.clone().unwrap())
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            return WorkflowExecutionResult::Failed {
                error: format!("{}: {}", t!("error.llm_error"), e),
                completed_steps: 0,
            };
        }
    };
    let instruction = match parse_plan_response(&llm_response) {
        Ok(instr) => instr,
        Err(e) => {
            return WorkflowExecutionResult::Failed {
                error: format!("Failed to parse plan: {}", e),
                completed_steps: 0,
            };
        }
    };

    match instruction {
        PlanInstruction {
            mode,
            plan,
            message,
        } => {
            if mode == "done" {
                let final_msg =
                    message.unwrap_or_else(|| t!("skill.no_actions_executed").to_string());
                return WorkflowExecutionResult::Completed(final_msg);
            }
            if let Some(plan) = plan {
                match execute_workflow_plan(executor, &plan, task_id.as_deref()).await {
                    Ok((result, success_count, failed_count)) => {
                        let raw_json = serde_json::json!({
                            "mode": "plan_and_execute",
                            "result": result,
                            "success_count": success_count,
                            "failed_count": failed_count,
                        })
                        .to_string();
                        WorkflowExecutionResult::CompletedWithRaw {
                            display: result,
                            raw_json,
                        }
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        WorkflowExecutionResult::Failed {
                            error: format!("Workflow failed: {}", error_msg),
                            completed_steps: 0,
                        }
                    }
                }
            } else {
                WorkflowExecutionResult::Completed(t!("skill.no_actions_executed").to_string())
            }
        }
    }
}
