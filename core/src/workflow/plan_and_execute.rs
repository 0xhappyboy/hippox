//! Plan-and-Execute mode workflow execution
//!
//! This mode generates a complete execution plan upfront and then executes it step by step.
//! It supports DAG-style workflows with conditional logic, variable references, and error handling.
//!
//! # Error Handling Strategy
//!
//! Since PlanAndExecute is a DAG (Directed Acyclic Graph) mode, node failures are handled
//! differently from other modes. Each node can define an `on_error` handler in the plan:
//!
//! - `on_error: "skip"` → After retries are exhausted, the failed node is skipped and the
//!   workflow continues with subsequent nodes. The failure is logged but does not block
//!   the rest of the workflow.
//!
//! - `on_error: "fail"` → After retries are exhausted, the entire workflow terminates
//!   immediately and returns a failure result. No further nodes are executed.
//!
//! - No `on_error` handler → The default behavior is the same as `"fail"`: the workflow
//!   terminates when a node fails after exhausting all retries.
//!
//! # Retry Behavior
//!
//! Each node in the plan inherits the global retry policy defined in `retry.rs`:
//! - `DEFAULT_MAX_RETRIES_PER_SKILL`: Maximum number of retry attempts for each node
//! - `DEFAULT_SKILL_TIMEOUT_SECS`: Timeout for each skill execution
//!
//! When a node fails, it will retry up to `max_retries` times before the error handler
//! determines the final outcome (skip or fail).
//!
//! # Node Dependencies
//!
//! Nodes are executed sequentially in the order defined in the plan. Each node can reference
//! outputs from previous nodes using the `{{variable_name}}` syntax. If a node is skipped
//! due to `on_error: "skip"`, its output is not available for subsequent nodes.
//!
//! # Example
//!
//! ```json
//! {
//!   "mode": "plan",
//!   "plan": {
//!     "steps": [
//!       {
//!         "id": "step1",
//!         "action": "file_read",
//!         "parameters": { "path": "/data/input.txt" },
//!         "output_as": "content",
//!         "on_error": { "action": "skip" }
//!       },
//!       {
//!         "id": "step2",
//!         "action": "file_write",
//!         "parameters": { "path": "/data/output.txt", "content": "{{content}}" },
//!         "on_error": { "action": "fail" }
//!       }
//!     ]
//!   }
//! }
//! ```
//!
//! In this example:
//! - If `step1` fails after retries, it is skipped and `step2` still executes (but `{{content}}` will be empty)
//! - If `step2` fails after retries, the entire workflow terminates

use crate::prompts::build_plan_prompt;
use crate::t;
use crate::{SkillScheduler, TASK_STEP_SIGNAL_BUS, check_task_interruption};
use hippox_atomic_skills::{Executor, SkillCall, SkillCallback, SkillContext};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

use super::core::WorkflowExecutor;
use super::retry::*;
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

/// Execute a single plan step with retry and timeout
async fn execute_plan_step_with_retry(
    executor: &Executor,
    call: SkillCall,
    step_name: String,
    step_id: &str,
    step_index: usize,
    task_id: Option<String>,
    workflow_callback: &Option<Arc<dyn WorkflowCallback>>,
    skill_callback_arc: Option<Arc<dyn SkillCallback>>,
    max_retries: usize,
    timeout_secs: u64,
    on_error_action: Option<&str>,
) -> anyhow::Result<String> {
    let mut retry_context = RetryContext::new(max_retries, DEFAULT_MAX_CONSECUTIVE_FAILURES);
    let mut last_error = None;
    loop {
        let skill_context = SkillContext {
            task_id: task_id.clone(),
            skill_index: Some(step_index),
            skill_name: Some(step_name.clone()),
            extra: HashMap::new(),
            signal_bus: Some(&TASK_STEP_SIGNAL_BUS),
        };
        let result = execute_skill_with_timeout(
            executor,
            &call,
            skill_callback_arc.clone(),
            Some(&skill_context),
            timeout_secs,
        )
        .await;
        match result {
            SkillExecutionResult::Success(output) => {
                return Ok(output);
            }
            SkillExecutionResult::Timeout(ref error_msg)
            | SkillExecutionResult::Failure(ref error_msg) => {
                let is_timeout = result.is_timeout();
                if let Some(cb) = workflow_callback {
                    if let Some(ref tid) = task_id {
                        if is_timeout {
                            cb.on_step_timeout(tid, &step_name, step_index, error_msg, 0)
                                .await;
                        } else {
                            cb.on_step_failure(tid, &step_name, step_index, error_msg, 0)
                                .await;
                        }
                    }
                }
                last_error = Some(error_msg.clone());
                if retry_context.can_retry(&step_name) {
                    info!(
                        "Retrying plan step '{}' (attempt {}/{})",
                        step_name,
                        retry_context.get_retry_count(&step_name),
                        max_retries
                    );
                    continue;
                } else {
                    // Check if we should skip on error
                    if let Some(action) = on_error_action {
                        if action == "skip" {
                            info!(
                                "Skipping plan step '{}' after {} retries",
                                step_name, max_retries
                            );
                            return Err(anyhow::anyhow!(
                                "SKIPPED: {}",
                                last_error.unwrap_or_default()
                            ));
                        }
                    }
                    return Err(anyhow::anyhow!(
                        "Skill '{}' failed after {} retries: {}",
                        step_name,
                        max_retries,
                        last_error.unwrap_or_default()
                    ));
                }
            }
        }
    }
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
    let skill_callback_arc: Option<Arc<dyn SkillCallback>> = executor.get_skill_callback();
    let timeout_secs = get_timeout_secs(executor);
    let max_retries = DEFAULT_MAX_RETRIES_PER_SKILL;
    for (idx, step) in plan.steps.iter().enumerate() {
        let checkpoint = serde_json::to_string(&WorkflowCheckpoint {
            last_completed_step: idx,
            variables: context.variables.clone(),
            completed_results: vec![],
            mode: WorkflowMode::PlanAndExecute,
            metadata: HashMap::new(),
        })
        .ok();
        // Check interruption
        match check_task_interruption(
            task_id,
            executor.get_workflow_callback(),
            idx,
            &step.action,
            checkpoint.clone(),
        )
        .await
        {
            Ok(_) => {}
            Err(result) => {
                return Err(anyhow::anyhow!("{:?}", result));
            }
        }
        // Check condition
        if let Some(condition) = &step.condition {
            if !evaluate_condition(condition, &context) {
                info!("Step {} condition not met, skipping", step.id);
                continue;
            }
        }
        // Resolve parameters
        let mut resolved_params = HashMap::new();
        for (key, value_ref) in &step.parameters {
            let resolved = resolve_value_ref(value_ref, &context);
            let final_resolved = resolve_variables_deep(&resolved, &string_context);
            resolved_params.insert(key.clone(), final_resolved);
        }
        let call = SkillCall {
            action: step.action.clone(),
            parameters: resolved_params,
        };
        let on_error_action = step.on_error.as_ref().map(|e| e.action.as_str());
        // Execute with retry
        match execute_plan_step_with_retry(
            executor.get_executor(),
            call,
            step.action.clone(),
            &step.id,
            idx,
            task_id.map(|s| s.to_string()),
            executor.get_workflow_callback(),
            skill_callback_arc.clone(),
            max_retries,
            timeout_secs,
            on_error_action,
        )
        .await
        {
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
                let error_msg = e.to_string();
                // Check if this is a "skip" error
                if error_msg.starts_with("SKIPPED:") {
                    // Step was skipped after retries - continue chain
                    let actual_error = error_msg.trim_start_matches("SKIPPED:").trim();
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
                        error: Some(actual_error.to_string()),
                    });
                    failed_count += 1;
                    info!(
                        "Plan step '{}' skipped after retries, continuing chain",
                        step.id
                    );
                    continue;
                }
                // Check if this step has on_error handling
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
                            info!(
                                "Plan step '{}' skipped via error handler, continuing chain",
                                step.id
                            );
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
                    // No error handler, fail the entire workflow
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
                        if error_msg.contains("Cancelled") {
                            return WorkflowExecutionResult::Cancelled { completed_steps: 0 };
                        } else if error_msg.contains("Paused") {
                            return WorkflowExecutionResult::Paused {
                                checkpoint: None,
                                completed_steps: 0,
                                partial_output: String::new(),
                            };
                        }
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
