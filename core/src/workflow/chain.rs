//! Chain mode workflow execution

use crate::SkillScheduler;
use crate::prompts::build_chain_prompt;
use crate::t;
use hippox_atomic_skills::{SkillCall, SkillCallback, SkillContext};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use super::core::WorkflowExecutor;
use super::types::*;
use super::utils::{VARIABLE_REGEX, format_step_results};

pub fn parse_chain_response(response: &str) -> anyhow::Result<ChainPlan> {
    let json_str = WorkflowExecutor::extract_json(response);
    let value: Value = serde_json::from_str(&json_str)?;

    #[derive(serde::Deserialize)]
    struct ChainStep {
        action: String,
        parameters: HashMap<String, Value>,
        output_as: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct ChainResponse {
        mode: String,
        steps: Vec<ChainStep>,
    }

    let chain: ChainResponse = serde_json::from_value(value)?;
    if chain.mode != "chain" {
        anyhow::bail!("Invalid chain mode: expected 'chain', got '{}'", chain.mode);
    }

    let steps = chain
        .steps
        .into_iter()
        .map(|s| ChainStepDef {
            action: s.action,
            parameters: s.parameters,
            output_as: s.output_as,
        })
        .collect();

    Ok(ChainPlan { steps })
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

async fn check_step_interruption(
    task_id: Option<&str>,
    callback: &Option<Arc<dyn WorkflowCallback>>,
    step_index: usize,
    step_name: &str,
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

pub async fn execute_chain(
    executor: &WorkflowExecutor,
    scheduler: &SkillScheduler,
    input: &str,
) -> WorkflowExecutionResult {
    let overall_start = Instant::now();
    let task_id = executor.get_task_id().map(|s| s.to_string());

    // Check for checkpoint to resume from
    let mut checkpoint_restored = false;
    let mut restored_context: HashMap<String, Value> = HashMap::new();
    let mut restored_results: Vec<StepResult> = Vec::new();
    let mut last_completed_idx = 0;

    if let Some(ref tid) = task_id {
        if let Some(state_updater) = crate::tasks::get_state_updater(tid).await {
            if let Some(checkpoint_data) = state_updater.get_checkpoint().await {
                if let Ok(checkpoint) = serde_json::from_str::<WorkflowCheckpoint>(&checkpoint_data)
                {
                    restored_context = checkpoint.variables;
                    restored_results = checkpoint.completed_results;
                    last_completed_idx = checkpoint.last_completed_step;
                    checkpoint_restored = true;
                    if let Some(cb) = executor.get_workflow_callback() {
                        cb.on_workflow_resumed(
                            tid,
                            overall_start.elapsed().as_millis() as u64,
                            restored_results.len(),
                        )
                        .await;
                    }
                }
            }
        }
    }
    if let Some(ref tid) = task_id {
        if let Some(state_updater) = crate::tasks::get_state_updater(tid).await {
            if state_updater.is_cancelled().await {
                if let Some(cb) = executor.get_workflow_callback() {
                    cb.on_workflow_cancelled(tid, 0, 0).await;
                }
                return WorkflowExecutionResult::Cancelled { completed_steps: 0 };
            }
            if state_updater.is_paused().await {
                if let Some(cb) = executor.get_workflow_callback() {
                    cb.on_workflow_paused(tid, None, 0, 0).await;
                }
                return WorkflowExecutionResult::Paused {
                    checkpoint: None,
                    completed_steps: 0,
                    partial_output: String::new(),
                };
            }
        }
    }
    let chain_prompt = build_chain_prompt(input);
    let llm_response = match scheduler
        .generate_with_task(&chain_prompt, &task_id.clone().unwrap())
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
    if let Some(ref tid) = task_id {
        if let Some(state_updater) = crate::tasks::get_state_updater(tid).await {
            if state_updater.is_cancelled().await {
                if let Some(cb) = executor.get_workflow_callback() {
                    cb.on_workflow_cancelled(tid, overall_start.elapsed().as_millis() as u64, 0)
                        .await;
                }
                return WorkflowExecutionResult::Cancelled { completed_steps: 0 };
            }
            if state_updater.is_paused().await {
                if let Some(cb) = executor.get_workflow_callback() {
                    cb.on_workflow_paused(tid, None, overall_start.elapsed().as_millis() as u64, 0)
                        .await;
                }
                return WorkflowExecutionResult::Paused {
                    checkpoint: None,
                    completed_steps: 0,
                    partial_output: String::new(),
                };
            }
        }
    }

    let chain = match parse_chain_response(&llm_response) {
        Ok(chain) => chain,
        Err(e) => {
            return WorkflowExecutionResult::Failed {
                error: format!("Failed to parse chain: {}", e),
                completed_steps: 0,
            };
        }
    };

    let mut context = if checkpoint_restored {
        restored_context
    } else {
        let mut new_context = HashMap::new();
        new_context.insert("user_input".to_string(), Value::String(input.to_string()));
        new_context
    };
    let mut results = restored_results;
    let start_idx = last_completed_idx;

    for (idx, step) in chain.steps.iter().enumerate().skip(start_idx) {
        let step_name = step.action.clone();
        let step_start = Instant::now();

        let checkpoint = serde_json::to_string(&WorkflowCheckpoint {
            last_completed_step: idx,
            variables: context.clone(),
            completed_results: results.clone(),
            mode: WorkflowMode::Chain,
            metadata: HashMap::new(),
        })
        .ok();

        if let Err(result) = check_step_interruption(
            task_id.as_deref(),
            executor.get_workflow_callback(),
            idx,
            &step_name,
            checkpoint.clone(),
        )
        .await
        {
            return result;
        }
        let mut resolved_params = HashMap::new();
        for (key, value) in &step.parameters {
            let resolved = resolve_variables_deep(value, &context);
            resolved_params.insert(key.clone(), resolved);
        }
        if let Some(cb) = executor.get_workflow_callback() {
            if let Some(ref tid) = task_id {
                cb.on_step_start(tid, &step_name, idx, Some(&resolved_params))
                    .await;
            }
        }
        let call = SkillCall {
            action: step.action.clone(),
            parameters: resolved_params,
        };
        let skill_context = SkillContext {
            task_id: task_id.clone(),
            skill_index: Some(idx),
            skill_name: Some(step.action.clone()),
            extra: HashMap::new(),
            signal_bus: None,
        };
        let skill_callback_arc: Option<Arc<dyn SkillCallback>> = executor.get_skill_callback();
        match executor
            .get_executor()
            .execute(&call, skill_callback_arc.as_deref(), Some(&skill_context))
            .await
        {
            Ok(output) => {
                let duration = step_start.elapsed().as_millis() as u64;
                if let Some(cb) = executor.get_workflow_callback() {
                    if let Some(ref tid) = task_id {
                        cb.on_step_success(tid, &step_name, idx, &output, duration)
                            .await;
                    }
                }
                if let Some(output_as) = &step.output_as {
                    if let Ok(num) = output.parse::<f64>() {
                        context.insert(output_as.clone(), json!(num));
                    } else {
                        context.insert(output_as.clone(), Value::String(output.clone()));
                    }
                }
                results.push(StepResult {
                    skill: step.action.clone(),
                    parameters: call.parameters,
                    output: output.clone(),
                    status: ExecutionStatus::Success,
                });
            }
            Err(e) => {
                let duration = step_start.elapsed().as_millis() as u64;
                let error_msg = e.to_string();
                if let Some(cb) = executor.get_workflow_callback() {
                    if let Some(ref tid) = task_id {
                        cb.on_step_failure(tid, &step_name, idx, &error_msg, duration)
                            .await;
                    }
                }
                results.push(StepResult {
                    skill: step.action.clone(),
                    parameters: call.parameters,
                    output: error_msg.clone(),
                    status: ExecutionStatus::Failure,
                });
                if let Some(cb) = executor.get_workflow_callback() {
                    if let Some(ref tid) = task_id {
                        let total_duration = overall_start.elapsed().as_millis() as u64;
                        cb.on_workflow_failed(tid, &error_msg, total_duration, results.len())
                            .await;
                    }
                }
                return WorkflowExecutionResult::Failed {
                    error: format!("Skill '{}' failed: {}", step.action, error_msg),
                    completed_steps: results.len(),
                };
            }
        }
    }
    let final_display = format_step_results(&results);
    let raw_json = serde_json::json!({
        "mode": "chain",
        "steps": results.iter().map(|r| {
            serde_json::json!({
                "skill": r.skill,
                "output": r.output,
                "status": match r.status {
                    ExecutionStatus::Success => "success",
                    ExecutionStatus::Failure => "failure",
                }
            })
        }).collect::<Vec<_>>()
    })
    .to_string();
    WorkflowExecutionResult::CompletedWithRaw {
        display: final_display,
        raw_json,
    }
}

pub async fn execute_chain_with_categories(
    executor: &WorkflowExecutor,
    scheduler: &SkillScheduler,
    input: &str,
    categories: &[String],
) -> WorkflowExecutionResult {
    let overall_start = Instant::now();
    let task_id = executor.get_task_id().map(|s| s.to_string());
    let filtered_skills = crate::prompts::generate_skills_registry_by_categories(categories);
    let chain_prompt = crate::prompts::build_chain_prompt_with_categories(&filtered_skills, input);
    let llm_response = match scheduler
        .generate_with_task(&chain_prompt, &task_id.clone().unwrap())
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
    let chain = match parse_chain_response(&llm_response) {
        Ok(chain) => chain,
        Err(e) => {
            return WorkflowExecutionResult::Failed {
                error: format!("Failed to parse chain: {}", e),
                completed_steps: 0,
            };
        }
    };

    let mut context = HashMap::new();
    context.insert("user_input".to_string(), Value::String(input.to_string()));
    let mut results = Vec::new();

    for (idx, step) in chain.steps.iter().enumerate() {
        let step_name = step.action.clone();
        let step_start = Instant::now();
        let mut resolved_params = HashMap::new();
        for (key, value) in &step.parameters {
            let resolved = resolve_variables_deep(value, &context);
            resolved_params.insert(key.clone(), resolved);
        }
        let call = SkillCall {
            action: step.action.clone(),
            parameters: resolved_params,
        };
        let skill_callback_arc: Option<Arc<dyn SkillCallback>> = executor.get_skill_callback();
        let skill_context = SkillContext {
            task_id: task_id.clone(),
            skill_index: Some(idx),
            skill_name: Some(step_name.clone()),
            extra: HashMap::new(),
            signal_bus: None,
        };
        match executor
            .get_executor()
            .execute(&call, skill_callback_arc.as_deref(), Some(&skill_context))
            .await
        {
            Ok(output) => {
                if let Some(output_as) = &step.output_as {
                    if let Ok(num) = output.parse::<f64>() {
                        context.insert(output_as.clone(), json!(num));
                    } else {
                        context.insert(output_as.clone(), Value::String(output.clone()));
                    }
                }
                results.push(StepResult {
                    skill: step.action.clone(),
                    parameters: call.parameters,
                    output: output.clone(),
                    status: ExecutionStatus::Success,
                });
            }
            Err(e) => {
                let error_msg = e.to_string();
                results.push(StepResult {
                    skill: step.action.clone(),
                    parameters: call.parameters,
                    output: error_msg.clone(),
                    status: ExecutionStatus::Failure,
                });
                return WorkflowExecutionResult::Failed {
                    error: format!("Skill '{}' failed: {}", step.action, error_msg),
                    completed_steps: results.len(),
                };
            }
        }
    }

    let final_display = format_step_results(&results);
    let raw_json = serde_json::json!({
        "mode": "chain",
        "steps": results.iter().map(|r| {
            serde_json::json!({
                "skill": r.skill,
                "output": r.output,
                "status": match r.status {
                    ExecutionStatus::Success => "success",
                    ExecutionStatus::Failure => "failure",
                }
            })
        }).collect::<Vec<_>>()
    })
    .to_string();

    WorkflowExecutionResult::CompletedWithRaw {
        display: final_display,
        raw_json,
    }
}
