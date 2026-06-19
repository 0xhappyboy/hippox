//! Batch mode workflow execution
//!
//! This mode executes multiple independent skills in parallel. Each skill is executed
//! with its own retry and timeout policy. Failures in one skill do not affect others.
//!
//! # Characteristics
//! - All skills are executed concurrently using `tokio::spawn`
//! - Each skill has independent retry (3 attempts) and timeout (60s) protection
//! - Results are collected and aggregated regardless of individual failures
//! - Best for: Bulk operations, independent tasks, parallel processing
//!
//! # Execution Flow
//! 1. LLM generates a batch plan containing multiple skill calls
//! 2. Each skill call is spawned as a separate tokio task
//! 3. Each task executes with its own retry context
//! 4. All results are collected and returned as a single batch result
//!
//! # Retry Behavior
//! Each skill in the batch inherits the global retry policy:
//! - Up to 3 retry attempts per skill
//! - 60-second timeout per execution attempt
//! - Individual skills do not affect each other's retry state

use crate::prompts::build_batch_prompt;
use crate::{
    SkillScheduler, TASK_STEP_SIGNAL_BUS, check_task_interruption, parse_react_response, t,
};
use futures::future::join_all;
use hippox_atomic_skills::{Executor, SkillCall, SkillCallback, SkillContext};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use super::core::WorkflowExecutor;
use super::retry::*;
use super::types::*;
use super::utils::format_step_results;

/// Execute a single skill with retry and timeout protection in batch mode.
///
/// This function handles the complete lifecycle of a single batch task:
/// - Executes the skill with timeout protection
/// - Automatically retries on failure or timeout (up to max_retries)
/// - Triggers appropriate callbacks for each attempt
/// - Returns either a success or failure StepResult
///
/// # Arguments
/// * `executor` - The skill executor
/// * `step` - The skill call to execute
/// * `step_name` - Name of the skill (for logging and callbacks)
/// * `idx` - Index of this step in the batch
/// * `task_id` - Optional task ID for task tracking
/// * `callback` - Optional workflow callback for progress notifications
/// * `skill_callback` - Optional skill callback for skill-level events
/// * `max_retries` - Maximum number of retry attempts
/// * `timeout_secs` - Timeout in seconds for each execution attempt
async fn execute_batch_skill_with_retry(
    executor: &Executor,
    step: SkillCall,
    step_name: String,
    idx: usize,
    task_id: Option<String>,
    callback: &Option<Arc<dyn WorkflowCallback>>,
    skill_callback: Option<Arc<dyn SkillCallback>>,
    max_retries: usize,
    timeout_secs: u64,
) -> StepResult {
    let step_start = Instant::now();
    let mut last_error = None;
    let mut retry_context = RetryContext::new(max_retries, DEFAULT_MAX_CONSECUTIVE_FAILURES);
    // Execute with retry
    loop {
        let call = step.clone();
        let skill_context = SkillContext {
            task_id: task_id.clone(),
            skill_index: Some(idx),
            skill_name: Some(step_name.clone()),
            extra: HashMap::new(),
            signal_bus: Some(&TASK_STEP_SIGNAL_BUS),
        };
        let result = execute_skill_with_timeout(
            executor,
            &call,
            skill_callback.clone(),
            Some(&skill_context),
            timeout_secs,
        )
        .await;
        match result {
            SkillExecutionResult::Success(output) => {
                let duration = step_start.elapsed().as_millis() as u64;
                if let Some(cb) = callback {
                    if let Some(ref tid) = task_id {
                        cb.on_step_success(tid, &step_name, idx, &output, duration)
                            .await;
                    }
                }
                return StepResult {
                    skill: step.action.clone(),
                    parameters: step.parameters.clone(),
                    output,
                    status: ExecutionStatus::Success,
                };
            }
            SkillExecutionResult::Timeout(ref error_msg)
            | SkillExecutionResult::Failure(ref error_msg) => {
                let is_timeout = result.is_timeout();
                let duration = step_start.elapsed().as_millis() as u64;
                let retry_count = retry_context.get_retry_count(&step_name);
                // Notify callback
                if let Some(cb) = callback {
                    if let Some(ref tid) = task_id {
                        if is_timeout {
                            cb.on_step_timeout(tid, &step_name, idx, &error_msg, duration)
                                .await;
                        } else {
                            cb.on_step_failure(tid, &step_name, idx, &error_msg, duration)
                                .await;
                        }
                    }
                }
                last_error = Some(error_msg.clone());
                // Check if we can retry
                if retry_context.can_retry(&step_name) {
                    // Continue to next retry
                    continue;
                } else {
                    // Max retries exceeded, return failure
                    return StepResult {
                        skill: step.action.clone(),
                        parameters: step.parameters.clone(),
                        output: format!(
                            "Failed after {} retries: {}",
                            max_retries,
                            last_error.unwrap_or_default()
                        ),
                        status: ExecutionStatus::Failure,
                    };
                }
            }
        }
    }
}

/// Execute a batch plan by running all skills in parallel.
///
/// Each skill in the batch is executed as an independent tokio task.
/// The function waits for all tasks to complete and collects their results.
///
/// # Arguments
/// * `executor` - The workflow executor
/// * `steps` - Slice of skill calls to execute in parallel
///
/// # Returns
/// A vector of StepResult for all executed skills
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
    if let Err(_) =
        check_task_interruption(task_id.as_deref(), &callback, 0, "batch_plan", None).await
    {
        return Vec::new();
    }
    let step_metadata: Vec<(usize, String)> = steps
        .iter()
        .enumerate()
        .map(|(idx, step)| (idx, step.action.clone()))
        .collect();
    let skill_callback_arc: Option<Arc<dyn SkillCallback>> = executor.get_skill_callback();
    let timeout_secs = get_timeout_secs(executor);
    let max_retries = DEFAULT_MAX_RETRIES_PER_SKILL;
    let futures = step_metadata.into_iter().map(|(idx, step_name)| {
        let step = steps[idx].clone();
        let executor = executor_clone.clone();
        let callback = callback.clone();
        let task_id = task_id.clone();
        let skill_callback = skill_callback_arc.clone();
        tokio::spawn(async move {
            if let Err(_) =
                check_task_interruption(task_id.as_deref(), &callback, idx, &step_name, None).await
            {
                return None;
            }
            let result = execute_batch_skill_with_retry(
                &executor,
                step,
                step_name,
                idx,
                task_id,
                &callback,
                skill_callback,
                max_retries,
                timeout_secs,
            )
            .await;
            Some(result)
        })
    });
    let results = join_all(futures).await;
    results
        .into_iter()
        .filter_map(|r| r.ok().flatten())
        .collect()
}

/// Execute a batch workflow with category filtering.
///
/// This is the main entry point for batch mode execution. It:
/// 1. Generates a batch plan using LLM with filtered skills
/// 2. Executes the plan in parallel
/// 3. Returns aggregated results
///
/// # Arguments
/// * `executor` - The workflow executor
/// * `scheduler` - The skill scheduler for LLM interactions
/// * `input` - User input text
/// * `categories` - Skill categories to filter by
///
/// # Returns
/// A WorkflowExecutionResult containing the batch results
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
