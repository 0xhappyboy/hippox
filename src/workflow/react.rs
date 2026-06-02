//! ReAct mode workflow execution

use crate::executors::SkillCall;
use crate::skill_scheduler::SkillScheduler;
use crate::t;
use langhub::types::ChatMessage;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use super::batch::execute_batch_plan;
use super::core::WorkflowExecutor;
use super::prompt;
use super::types::*;
use super::utils::format_step_results;

pub fn parse_react_response(response: &str) -> anyhow::Result<ReactInstruction> {
    let json_str = WorkflowExecutor::extract_json(response);
    let value: Value = serde_json::from_str(&json_str)?;
    if let Some(message) = value.get("message").and_then(|v| v.as_str()) {
        if value.get("action").and_then(|v| v.as_str()) == Some("done") {
            return Ok(ReactInstruction::Done(message.to_string()));
        }
    }
    if let Some(mode) = value.get("mode").and_then(|v| v.as_str()) {
        if mode == "batch" {
            if let Some(steps) = value.get("steps").and_then(|v| v.as_array()) {
                let mut skill_calls = Vec::new();
                for step in steps {
                    let call: SkillCall = serde_json::from_value(step.clone())?;
                    skill_calls.push(call);
                }
                return Ok(ReactInstruction::Batch(skill_calls));
            }
        }
    }
    if let Ok(call) = serde_json::from_value(value) {
        return Ok(ReactInstruction::Single(call));
    }
    anyhow::bail!("Unable to parse LLM response: {}", response)
}

async fn check_task_interruption(
    task_id: Option<&str>,
    callback: &Option<Arc<dyn WorkflowCallback>>,
    step_index: usize,
    step_name: &str,
    checkpoint: Option<String>,
) -> Result<(), WorkflowExecutionResult> {
    if let Some(tid) = task_id {
        if let Some(cb) = callback {
            if let Some(state_updater) = crate::tasks::get_state_updater(tid).await {
                if state_updater.is_cancelled().await {
                    let info = StepInterruptionInfo {
                        interrupted: true,
                        reason: "cancelled".to_string(),
                        step_index,
                        step_name: step_name.to_string(),
                        checkpoint: checkpoint.clone(),
                    };
                    cb.on_step_interrupted(tid, &info).await;
                    cb.on_workflow_cancelled(tid, 0, step_index).await;
                    return Err(WorkflowExecutionResult::Cancelled {
                        completed_steps: step_index,
                    });
                }
                if state_updater.is_paused().await {
                    if let Some(ref checkpoint_data) = checkpoint {
                        state_updater.save_checkpoint(checkpoint_data).await;
                    }
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
                    return Err(WorkflowExecutionResult::Paused {
                        checkpoint: checkpoint.clone(),
                        completed_steps: step_index,
                        partial_output: String::new(),
                    });
                }
            }
        }
    }
    Ok(())
}

pub async fn execute_react(
    executor: &WorkflowExecutor,
    scheduler: &SkillScheduler,
    input: &str,
    skills_registry: &str,
    instances_registry: &str,
) -> WorkflowExecutionResult {
    let overall_start = Instant::now();
    let input_trimmed = input.trim();
    if input_trimmed == "clear" {
        return WorkflowExecutionResult::Completed(t!("app.conversation_cleared").to_string());
    }
    if input_trimmed == "exit" || input_trimmed == "quit" {
        return WorkflowExecutionResult::Completed("goodbye".to_string());
    }
    if input_trimmed.is_empty() {
        return WorkflowExecutionResult::Completed(String::new());
    }
    let mut step_results: Vec<StepResult> = Vec::new();
    let mut final_response = None;
    let mut iteration = 0;
    let mut messages: Vec<ChatMessage> = Vec::new();
    let system_prompt = prompt::build_react_prompt(skills_registry, instances_registry);
    messages.push(ChatMessage::system(&system_prompt));
    messages.push(ChatMessage::user(input_trimmed));
    let task_id = executor.get_task_id().map(|s| s.to_string());

    while iteration < executor.max_iterations {
        iteration += 1;

        if let Some(ref tid) = task_id {
            if let Some(state_updater) = crate::tasks::get_state_updater(tid).await {
                if state_updater.is_cancelled().await {
                    if let Some(cb) = executor.get_callback() {
                        cb.on_workflow_cancelled(
                            tid,
                            overall_start.elapsed().as_millis() as u64,
                            step_results.len(),
                        )
                        .await;
                    }
                    return WorkflowExecutionResult::Cancelled {
                        completed_steps: step_results.len(),
                    };
                }
                if state_updater.is_paused().await {
                    if let Some(cb) = executor.get_callback() {
                        let checkpoint = serde_json::to_string(&WorkflowCheckpoint {
                            last_completed_step: step_results.len(),
                            variables: HashMap::new(),
                            completed_results: step_results.clone(),
                            mode: WorkflowMode::ReAct,
                            metadata: HashMap::new(),
                        })
                        .ok();
                        if let Some(ref checkpoint_data) = checkpoint {
                            state_updater.save_checkpoint(checkpoint_data).await;
                        }
                        cb.on_workflow_paused(
                            tid,
                            checkpoint.as_deref(),
                            overall_start.elapsed().as_millis() as u64,
                            step_results.len(),
                        )
                        .await;
                    }
                    return WorkflowExecutionResult::Paused {
                        checkpoint: None,
                        completed_steps: step_results.len(),
                        partial_output: format_step_results(&step_results),
                    };
                }
            }
        }

        let llm_response = match scheduler.get_llm().chat(messages.clone()).await {
            Ok(resp) => resp,
            Err(e) => {
                return WorkflowExecutionResult::Failed {
                    error: format!("{}: {}", t!("error.llm_error"), e),
                    completed_steps: step_results.len(),
                };
            }
        };
        messages.push(ChatMessage::assistant(&llm_response));
        let instruction = match parse_react_response(&llm_response) {
            Ok(instr) => instr,
            Err(_) => {
                return WorkflowExecutionResult::Completed(llm_response);
            }
        };
        match instruction {
            ReactInstruction::Done(message) => {
                final_response = Some(message);
                break;
            }
            ReactInstruction::Single(call) => {
                let step_index = step_results.len();
                let step_name = call.action.clone();
                let step_start = Instant::now();

                if let Err(result) = check_task_interruption(
                    task_id.as_deref(),
                    executor.get_callback(),
                    step_index,
                    &step_name,
                    None,
                )
                .await
                {
                    return result;
                }

                if let Some(cb) = executor.get_callback() {
                    if let Some(ref tid) = task_id {
                        cb.on_step_start(tid, &step_name, step_index, Some(&call.parameters))
                            .await;
                    }
                }

                match executor.get_executor().execute(&call).await {
                    Ok(output) => {
                        let duration = step_start.elapsed().as_millis() as u64;
                        if let Some(cb) = executor.get_callback() {
                            if let Some(ref tid) = task_id {
                                cb.on_step_success(tid, &step_name, step_index, &output, duration)
                                    .await;
                            }
                        }
                        step_results.push(StepResult {
                            skill: call.action.clone(),
                            parameters: call.parameters.clone(),
                            output: output.clone(),
                            status: ExecutionStatus::Success,
                        });
                        messages.push(ChatMessage::user(&format!(
                            "Skill '{}' executed. Result: {}",
                            call.action, output
                        )));
                    }
                    Err(e) => {
                        let duration = step_start.elapsed().as_millis() as u64;
                        let error_msg = e.to_string();
                        if let Some(cb) = executor.get_callback() {
                            if let Some(ref tid) = task_id {
                                cb.on_step_failure(
                                    tid, &step_name, step_index, &error_msg, duration,
                                )
                                .await;
                            }
                        }
                        step_results.push(StepResult {
                            skill: call.action.clone(),
                            parameters: call.parameters.clone(),
                            output: error_msg.clone(),
                            status: ExecutionStatus::Failure,
                        });
                        final_response = Some(format!(
                            "{} '{}': {}",
                            t!("error.skill_failed"),
                            call.action,
                            error_msg
                        ));
                        break;
                    }
                }
            }
            ReactInstruction::Batch(steps) => {
                let step_index = step_results.len();
                let step_name = format!("batch_{}_steps", steps.len());

                if let Err(result) = check_task_interruption(
                    task_id.as_deref(),
                    executor.get_callback(),
                    step_index,
                    &step_name,
                    None,
                )
                .await
                {
                    return result;
                }

                let results = execute_batch_plan(executor, &steps).await;
                let mut batch_output = String::new();
                for (i, result) in results.iter().enumerate() {
                    step_results.push(result.clone());
                    batch_output.push_str(&format!("Step {}: {}\n", i + 1, result.output));
                }
                messages.push(ChatMessage::user(&format!(
                    "Batch execution completed. Results:\n{}",
                    batch_output
                )));
                let summary = format_step_results(&step_results);
                final_response = Some(summary);
                break;
            }
        }
    }

    if iteration >= executor.max_iterations {
        final_response = Some(t!("error.max_iterations_reached").to_string());
    }

    let total_duration = overall_start.elapsed().as_millis() as u64;
    let final_response = final_response.unwrap_or_else(|| {
        if step_results.is_empty() {
            t!("skill.no_actions_executed").to_string()
        } else {
            format_step_results(&step_results)
        }
    });

    if let Some(cb) = executor.get_callback() {
        if let Some(ref tid) = task_id {
            let has_failure = step_results
                .iter()
                .any(|r| r.status == ExecutionStatus::Failure)
                || final_response.starts_with("Error:")
                || final_response.contains("failed");
            let total_steps = step_results.len();
            if has_failure {
                cb.on_workflow_failed(tid, &final_response, total_duration, total_steps)
                    .await;
            } else {
                cb.on_workflow_complete(tid, &final_response, total_duration, total_steps)
                    .await;
            }
        }
    }

    WorkflowExecutionResult::Completed(final_response)
}
