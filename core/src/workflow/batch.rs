//! Batch mode workflow execution
//!
//! This mode executes multiple independent drivers in parallel. Each driver is executed
//! with its own retry and timeout policy. Failures in one driver do not affect others.
//!
//! # Characteristics
//! - All drivers are executed concurrently using `tokio::spawn`
//! - Each driver has independent retry (3 attempts) and timeout (60s) protection
//! - Results are collected and aggregated regardless of individual failures
//! - Best for: Bulk operations, independent tasks, parallel processing
//!
//! # Execution Flow
//! 1. LLM generates a batch plan containing multiple driver calls
//! 2. Each driver call is spawned as a separate tokio task
//! 3. Each task executes with its own retry context
//! 4. All results are collected and returned as a single batch result
//!
//! # Retry Behavior
//! Each driver in the batch inherits the global retry policy:
//! - Up to 3 retry attempts per driver
//! - 60-second timeout per execution attempt
//! - Individual drivers do not affect each other's retry state

use crate::prompts::build_batch_prompt;
use crate::{
    DriverScheduler, TASK_STEP_SIGNAL_BUS, check_task_interruption, parse_react_response, t,
};
use futures::future::join_all;
use hippox_drivers::{Executor, DriverCall, DriverCallback, DriverContext};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use super::core::WorkflowExecutor;
use super::retry::*;
use super::types::*;
use super::utils::format_step_results;

/// Execute a single driver with retry and timeout protection in batch mode.
///
/// This function handles the complete lifecycle of a single batch task:
/// - Executes the driver with timeout protection
/// - Automatically retries on failure or timeout (up to max_retries)
/// - Triggers appropriate callbacks for each attempt
/// - Returns either a success or failure StepResult
///
/// # Arguments
/// * `executor` - The driver executor
/// * `step` - The driver call to execute
/// * `step_name` - Name of the driver (for logging and callbacks)
/// * `idx` - Index of this step in the batch
/// * `task_id` - Optional task ID for task tracking
/// * `callback` - Optional workflow callback for progress notifications
/// * `driver_callback` - Optional driver callback for driver-level events
/// * `max_retries` - Maximum number of retry attempts
/// * `timeout_secs` - Timeout in seconds for each execution attempt
async fn execute_batch_driver_with_retry(
    executor: &Executor,
    step: DriverCall,
    step_name: String,
    idx: usize,
    task_id: Option<String>,
    callback: &Option<Arc<dyn WorkflowCallback>>,
    driver_callback: Option<Arc<dyn DriverCallback>>,
    max_retries: usize,
    timeout_secs: u64,
) -> StepResult {
    let step_start = Instant::now();
    let mut last_error = None;
    let mut retry_context = RetryContext::new(max_retries, DEFAULT_MAX_CONSECUTIVE_FAILURES);
    // Execute with retry
    loop {
        let call = step.clone();
        let driver_context = DriverContext {
            task_id: task_id.clone(),
            driver_index: Some(idx),
            driver_name: Some(step_name.clone()),
            extra: HashMap::new(),
            signal_bus: Some(&TASK_STEP_SIGNAL_BUS),
        };
        let result = execute_driver_with_timeout(
            executor,
            &call,
            driver_callback.clone(),
            Some(&driver_context),
            timeout_secs,
        )
        .await;
        match result {
            DriverExecutionResult::Success(output) => {
                let duration = step_start.elapsed().as_millis() as u64;
                if let Some(cb) = callback {
                    if let Some(ref tid) = task_id {
                        cb.on_step_success(tid, &step_name, idx, &output, duration)
                            .await;
                    }
                }
                return StepResult {
                    driver: step.action.clone(),
                    parameters: step.parameters.clone(),
                    output,
                    status: ExecutionStatus::Success,
                };
            }
            DriverExecutionResult::Timeout(ref error_msg)
            | DriverExecutionResult::Failure(ref error_msg) => {
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
                        driver: step.action.clone(),
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

/// Execute a batch plan by running all drivers in parallel.
///
/// Each driver in the batch is executed as an independent tokio task.
/// The function waits for all tasks to complete and collects their results.
///
/// # Arguments
/// * `executor` - The workflow executor
/// * `steps` - Slice of driver calls to execute in parallel
///
/// # Returns
/// A vector of StepResult for all executed drivers
pub async fn execute_batch_plan(
    executor: &WorkflowExecutor,
    steps: &[DriverCall],
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
    let driver_callback_arc: Option<Arc<dyn DriverCallback>> = executor.get_driver_callback();
    let timeout_secs = get_timeout_secs(executor);
    let max_retries = DEFAULT_MAX_RETRIES_PER_SKILL;
    let futures = step_metadata.into_iter().map(|(idx, step_name)| {
        let step = steps[idx].clone();
        let executor = executor_clone.clone();
        let callback = callback.clone();
        let task_id = task_id.clone();
        let driver_callback = driver_callback_arc.clone();
        tokio::spawn(async move {
            if let Err(_) =
                check_task_interruption(task_id.as_deref(), &callback, idx, &step_name, None).await
            {
                return None;
            }
            let result = execute_batch_driver_with_retry(
                &executor,
                step,
                step_name,
                idx,
                task_id,
                &callback,
                driver_callback,
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
/// 1. Generates a batch plan using LLM with filtered drivers
/// 2. Executes the plan in parallel
/// 3. Returns aggregated results
///
/// # Arguments
/// * `executor` - The workflow executor
/// * `scheduler` - The driver scheduler for LLM interactions
/// * `input` - User input text
/// * `categories` - Driver categories to filter by
///
/// # Returns
/// A WorkflowExecutionResult containing the batch results
pub async fn execute_batch_with_categories(
    executor: &WorkflowExecutor,
    scheduler: &DriverScheduler,
    input: &str,
    categories: &[String],
) -> WorkflowExecutionResult {
    let overall_start = Instant::now();
    let task_id = executor.get_task_id().map(|s| s.to_string());
    let filtered_drivers = crate::prompts::generate_drivers_registry_by_categories(categories);
    let batch_prompt = crate::prompts::build_batch_prompt_with_categories(&filtered_drivers, input);
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
                        "driver": r.driver,
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
