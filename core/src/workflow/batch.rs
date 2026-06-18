//! Batch mode workflow execution

use crate::prompts::build_batch_prompt;
use crate::{SkillScheduler, TASK_STEP_SIGNAL_BUS, t};
use futures::future::join_all;
use hippox_atomic_skills::{SkillCall, SkillCallback, SkillContext};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use super::core::WorkflowExecutor;
use super::react::parse_react_response;
use super::types::*;
use super::utils::format_step_results;

pub async fn execute_batch_plan(
    executor: &WorkflowExecutor,
    steps: &[SkillCall],
) -> Vec<StepResult> {
    if steps.is_empty() {
        return Vec::new();
    }
    let callback = executor.get_workflow_callback().clone();
    let executor_clone = executor.get_executor().clone();
    let task_id = executor.get_task_id().map(|s| s.to_string());
    if let Some(ref tid) = task_id {
        if let Some(state_updater) = crate::tasks::get_state_updater(tid).await {
            if state_updater.is_cancelled().await {
                if let Some(cb) = &callback {
                    cb.on_workflow_cancelled(tid, 0, 0).await;
                }
                return Vec::new();
            }
            if state_updater.is_paused().await {
                if let Some(cb) = &callback {
                    let checkpoint = serde_json::to_string(&WorkflowCheckpoint {
                        last_completed_step: 0,
                        variables: HashMap::new(),
                        completed_results: vec![],
                        mode: WorkflowMode::Batch,
                        metadata: HashMap::new(),
                    })
                    .ok();
                    if let Some(ref checkpoint_data) = checkpoint {
                        state_updater.save_checkpoint(checkpoint_data).await;
                    }
                    cb.on_workflow_paused(tid, checkpoint.as_deref(), 0, 0)
                        .await;
                }
                return Vec::new();
            }
        }
    }
    let step_metadata: Vec<(usize, String)> = steps
        .iter()
        .enumerate()
        .map(|(idx, step)| (idx, step.action.clone()))
        .collect();
    let skill_callback_arc: Option<Arc<dyn SkillCallback>> = executor.get_skill_callback();
    let futures = step_metadata.into_iter().map(|(idx, step_name)| {
        let step = steps[idx].clone();
        let executor = executor_clone.clone();
        let callback = callback.clone();
        let task_id = task_id.clone();
        let skill_callback = skill_callback_arc.clone();
        tokio::spawn(async move {
            let step_start = Instant::now();
            if let Some(ref tid) = task_id {
                if let Some(state_updater) = crate::tasks::get_state_updater(tid).await {
                    if state_updater.is_cancelled().await {
                        if let Some(cb) = &callback {
                            let info = StepInterruptionInfo {
                                interrupted: true,
                                reason: "cancelled".to_string(),
                                step_index: idx,
                                step_name: step_name.clone(),
                                checkpoint: None,
                            };
                            cb.on_step_interrupted(tid, &info).await;
                        }
                        return None;
                    }
                    if state_updater.is_paused().await {
                        if let Some(cb) = &callback {
                            let info = StepInterruptionInfo {
                                interrupted: true,
                                reason: "paused".to_string(),
                                step_index: idx,
                                step_name: step_name.clone(),
                                checkpoint: None,
                            };
                            cb.on_step_interrupted(tid, &info).await;
                        }
                        return None;
                    }
                }
            }
            if let Some(cb) = &callback {
                if let Some(ref tid) = task_id {
                    cb.on_step_start(tid, &step_name, idx, Some(&step.parameters))
                        .await;
                }
            }
            // set skill context and callback
            let skill_context = SkillContext {
                task_id: task_id.clone(),
                skill_index: Some(idx),
                skill_name: Some(step_name.clone()),
                extra: HashMap::new(),
                signal_bus: Some(&TASK_STEP_SIGNAL_BUS),
            };
            match executor
                .execute(&step, skill_callback.as_deref(), Some(&skill_context))
                .await
            {
                Ok(output) => {
                    let duration = step_start.elapsed().as_millis() as u64;
                    if let Some(cb) = &callback {
                        if let Some(ref tid) = task_id {
                            cb.on_step_success(tid, &step_name, idx, &output, duration)
                                .await;
                        }
                    }
                    Some(StepResult {
                        skill: step.action.clone(),
                        parameters: step.parameters.clone(),
                        output,
                        status: ExecutionStatus::Success,
                    })
                }
                Err(e) => {
                    let duration = step_start.elapsed().as_millis() as u64;
                    let error_msg = format!("Failed: {}", e);
                    if let Some(cb) = &callback {
                        if let Some(ref tid) = task_id {
                            cb.on_step_failure(tid, &step_name, idx, &error_msg, duration)
                                .await;
                        }
                    }
                    Some(StepResult {
                        skill: step.action.clone(),
                        parameters: step.parameters.clone(),
                        output: error_msg,
                        status: ExecutionStatus::Failure,
                    })
                }
            }
        })
    });
    let results = join_all(futures).await;
    results
        .into_iter()
        .filter_map(|r| r.ok().flatten())
        .collect()
}

pub async fn execute_batch_with_categories(
    executor: &WorkflowExecutor,
    scheduler: &SkillScheduler,
    input: &str,
    categories: &[String],
) -> WorkflowExecutionResult {
    let overall_start = Instant::now();
    let task_id = executor.get_task_id().map(|s| s.to_string());
    let filtered_skills = crate::prompts::generate_skills_registry_by_categories(categories);
    let batch_prompt = crate::prompts::build_batch_prompt_with_categories(&filtered_skills, input);
    let task_id_str = task_id.as_deref().unwrap_or("unknown");
    let llm_response = match scheduler
        .generate_with_task(&batch_prompt, task_id_str)
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
    let instruction = match parse_react_response(&llm_response) {
        Ok(instr) => instr,
        Err(_) => {
            return WorkflowExecutionResult::Completed(llm_response);
        }
    };
    match instruction {
        ReactInstruction::Done(message) => WorkflowExecutionResult::Completed(message),
        ReactInstruction::Batch(steps) => {
            let results = execute_batch_plan(executor, &steps).await;
            let display = format_step_results(&results);
            let raw_json = serde_json::json!({
                "mode": "batch",
                "results": results.iter().map(|r| {
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
            WorkflowExecutionResult::CompletedWithRaw { display, raw_json }
        }
        ReactInstruction::Single(_) => {
            WorkflowExecutionResult::Completed(t!("error.batch_mode_invalid").to_string())
        }
    }
}
