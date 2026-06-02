//! Batch mode workflow execution

use crate::executors::{Executor, SkillCall};
use crate::skill_scheduler::SkillScheduler;
use crate::t;
use futures::future::join_all;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use super::core::WorkflowExecutor;
use super::prompt;
use super::react::parse_react_response;
use super::types::*;
use super::utils::format_step_results;

/// Execute batch plan
pub async fn execute_batch_plan(
    executor: &WorkflowExecutor,
    steps: &[SkillCall],
) -> Vec<StepResult> {
    if steps.is_empty() {
        return Vec::new();
    }
    let callback = executor.get_callback().clone();
    let executor_clone = executor.get_executor().clone();
    let task_id = executor.get_task_id().map(|s| s.to_string());
    let futures = steps.iter().enumerate().map(|(idx, step)| {
        let step = step.clone();
        let executor = executor_clone.clone();
        let callback = callback.clone();
        let task_id = task_id.clone();
        tokio::spawn(async move {
            let step_name = step.action.clone();
            let step_start = Instant::now();

            if let Some(cb) = &callback {
                if let Some(ref tid) = task_id {
                    cb.on_step_start(tid, &step_name, idx, Some(&step.parameters))
                        .await;
                }
            }
            match executor.execute(&step).await {
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

/// Execute batch mode workflow
pub async fn execute_batch(
    executor: &WorkflowExecutor,
    scheduler: &SkillScheduler,
    input: &str,
    skills_registry: &str,
    instances_registry: &str,
) -> String {
    let overall_start = Instant::now();
    let batch_prompt = prompt::build_batch_prompt(skills_registry, instances_registry, input);

    let llm_response = match scheduler.get_llm().generate(&batch_prompt).await {
        Ok(resp) => resp,
        Err(e) => return format!("{}: {}", t!("error.llm_error"), e),
    };
    let instruction = match parse_react_response(&llm_response) {
        Ok(instr) => instr,
        Err(_) => return llm_response,
    };
    let (result, total_steps) = match &instruction {
        ReactInstruction::Done(message) => (message.clone(), 0),
        ReactInstruction::Batch(steps) => {
            let results = execute_batch_plan(executor, steps).await;
            (format_step_results(&results), steps.len())
        }
        ReactInstruction::Single(_) => (t!("error.batch_mode_invalid").to_string(), 0),
    };
    let total_duration = overall_start.elapsed().as_millis() as u64;
    let task_id = executor.get_task_id().map(|s| s.to_string());
    if let Some(cb) = executor.get_callback() {
        if let Some(ref tid) = task_id {
            cb.on_workflow_complete(tid, &result, total_duration, total_steps)
                .await;
        }
    }
    result
}
