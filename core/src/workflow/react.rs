//! ReAct mode workflow execution
//!
//! This mode implements the ReAct (Reasoning + Acting) pattern where the LLM
//! iteratively decides which skill to execute based on previous results.
//! It is the most flexible and intelligent mode, suitable for open-ended tasks.
//!
//! # Characteristics
//! - LLM-driven decision making at each step
//! - Each skill execution has timeout (60s) and retry (3 attempts) protection
//! - Full error feedback loop: errors are sent back to LLM for decision
//! - LLM can retry, switch skills, or finish based on error context
//! - Best for: Open-ended tasks, dynamic decision making, error recovery
//!
//! # Execution Flow
//! 1. LLM receives the user input and skill registry
//! 2. LLM decides: execute a skill, execute a batch, or finish
//! 3. Skill is executed with timeout and retry protection
//! 4. Result (success or error) is fed back to LLM
//! 5. LLM decides the next action based on the result
//! 6. Loop continues until LLM decides to finish or max iterations reached
//!
//! # Retry Behavior
//! - Each skill has up to 3 retry attempts
//! - Retry decisions are made by LLM (not automatic)
//! - LLM receives structured error feedback to make informed decisions
//! - LLM can adjust parameters, switch skills, or abort

use crate::prompts::build_react_prompt;
use crate::{
    SkillScheduler, TASK_STEP_SIGNAL_BUS, check_task_interruption, parse_react_response, t,
};
use hippox_atomic_skills::{SkillCall, SkillCallback, SkillContext};
use langhub::types::ChatMessage;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use super::batch::execute_batch_plan;
use super::core::WorkflowExecutor;
use super::retry::*;
use super::types::*;
use super::utils::format_step_results;

/// Execute a ReAct workflow with category filtering.
///
/// This is the main entry point for ReAct mode execution. It implements the
/// full Think → Act → Observe loop with LLM-driven decision making.
///
/// # Arguments
/// * `executor` - The workflow executor
/// * `scheduler` - The skill scheduler for LLM interactions
/// * `input` - User input text
/// * `categories` - Skill categories to filter by
///
/// # Returns
/// A WorkflowExecutionResult containing the final response and execution history
pub async fn execute_react_with_categories(
    executor: &WorkflowExecutor,
    scheduler: &SkillScheduler,
    input: &str,
    categories: &[String],
) -> WorkflowExecutionResult {
    let overall_start = Instant::now();
    let input_trimmed = input.trim();
    if input_trimmed.is_empty() {
        return WorkflowExecutionResult::Completed(String::new());
    }
    let mut step_results: Vec<StepResult> = Vec::new();
    let mut final_response = None;
    let mut iteration = 0;
    let mut messages: Vec<ChatMessage> = Vec::new();
    // Initialize retry context with configured values
    let mut retry_context = RetryContext::new(
        DEFAULT_MAX_RETRIES_PER_SKILL,
        DEFAULT_MAX_CONSECUTIVE_FAILURES,
    );
    // Build filtered skills prompt
    let filtered_skills = crate::prompts::generate_skills_registry_by_categories(categories);
    let react_workflow_prompt =
        crate::prompts::build_react_prompt_with_categories(&filtered_skills);
    messages.push(ChatMessage::system(&react_workflow_prompt));
    messages.push(ChatMessage::user(input_trimmed));
    let task_id = executor.get_task_id().map(|s| s.to_string());
    // Check for checkpoint to resume from
    if let Some(ref tid) = task_id {
        if let Some(state_updater) = crate::tasks::get_state_updater(tid).await {
            if let Some(checkpoint_data) = state_updater.get_checkpoint().await {
                if let Ok(checkpoint) = serde_json::from_str::<WorkflowCheckpoint>(&checkpoint_data)
                {
                    step_results = checkpoint.completed_results;
                    for result in &step_results {
                        messages.push(ChatMessage::user(&format!(
                            "Skill '{}' executed. Result: {}",
                            result.skill, result.output
                        )));
                    }
                    if let Some(cb) = executor.get_workflow_callback() {
                        cb.on_workflow_resumed(
                            tid,
                            overall_start.elapsed().as_millis() as u64,
                            step_results.len(),
                        )
                        .await;
                    }
                }
            }
        }
    }
    while iteration < executor.max_iterations {
        iteration += 1;
        // Check consecutive failures threshold
        if retry_context.has_exceeded_consecutive_failures() {
            let warning_prompt = build_consecutive_failures_feedback(
                retry_context.consecutive_failures(),
                retry_context.max_consecutive_failures(),
            );
            messages.push(warning_prompt);
        }
        // Check task interruption (cancelled/paused)
        if let Some(ref tid) = task_id {
            if let Some(state_updater) = crate::tasks::get_state_updater(tid).await {
                if state_updater.is_cancelled().await {
                    if let Some(cb) = executor.get_workflow_callback() {
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
                    if let Some(cb) = executor.get_workflow_callback() {
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
        // Call LLM to get next instruction
        let llm_response = match scheduler
            .chat_with_task(messages.clone(), &task_id.clone().unwrap())
            .await
        {
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
                let step_name = call.action.clone();
                let step_index = step_results.len();
                // Check if this skill has permanently failed
                if retry_context.is_skill_permanently_failed(&step_name) {
                    let error_msg = format!(
                        "Skill '{}' has exceeded max retries ({}) and is permanently failed.",
                        step_name, DEFAULT_MAX_RETRIES_PER_SKILL
                    );
                    step_results.push(create_failed_step_result(&call, &error_msg));

                    let force_prompt = build_max_retries_exceeded_feedback(
                        &step_name,
                        DEFAULT_MAX_RETRIES_PER_SKILL,
                        &error_msg,
                    );
                    messages.push(force_prompt);
                    continue;
                }
                let step_start = Instant::now();
                // Check task interruption before execution
                if let Err(result) = check_task_interruption(
                    task_id.as_deref(),
                    executor.get_workflow_callback(),
                    step_index,
                    &step_name,
                    None,
                )
                .await
                {
                    return result;
                }
                // Trigger on_step_start callback
                if let Some(cb) = executor.get_workflow_callback() {
                    if let Some(ref tid) = task_id {
                        cb.on_step_start(tid, &step_name, step_index, Some(&call.parameters))
                            .await;
                    }
                }
                // Prepare skill execution context
                let skill_callback_arc: Option<Arc<dyn SkillCallback>> =
                    executor.get_skill_callback();
                let skill_context = SkillContext {
                    task_id: task_id.clone(),
                    skill_index: Some(step_index),
                    skill_name: Some(step_name.clone()),
                    extra: HashMap::new(),
                    signal_bus: Some(&TASK_STEP_SIGNAL_BUS),
                };
                // Execute skill with timeout
                let result = execute_skill_with_timeout(
                    executor.get_executor(),
                    &call,
                    skill_callback_arc,
                    Some(&skill_context),
                    DEFAULT_SKILL_TIMEOUT_SECS,
                )
                .await;
                match result {
                    SkillExecutionResult::Success(output) => {
                        retry_context.reset_consecutive_failures();
                        let duration = step_start.elapsed().as_millis() as u64;
                        if let Some(cb) = executor.get_workflow_callback() {
                            if let Some(ref tid) = task_id {
                                cb.on_step_success(tid, &step_name, step_index, &output, duration)
                                    .await;
                            }
                        }
                        step_results.push(create_success_step_result(&call, &output));
                        messages.push(ChatMessage::user(&format!(
                            "Skill '{}' executed successfully. Result: {}",
                            call.action, output
                        )));
                    }
                    SkillExecutionResult::Failure(ref error_msg)
                    | SkillExecutionResult::Timeout(ref error_msg) => {
                        retry_context.record_failure(&step_name);
                        let duration = step_start.elapsed().as_millis() as u64;
                        let retry_count = retry_context.get_retry_count(&step_name);
                        let is_timeout = result.is_timeout();
                        // Notify callback
                        if let Some(cb) = executor.get_workflow_callback() {
                            if let Some(ref tid) = task_id {
                                if is_timeout {
                                    cb.on_step_timeout(
                                        tid, &step_name, step_index, error_msg, duration,
                                    )
                                    .await;
                                } else {
                                    cb.on_step_failure(
                                        tid, &step_name, step_index, error_msg, duration,
                                    )
                                    .await;
                                }
                            }
                        }
                        step_results.push(create_step_result(&call, &result));
                        // Build appropriate feedback for LLM
                        let feedback = if is_timeout {
                            build_timeout_feedback(
                                &step_name,
                                DEFAULT_SKILL_TIMEOUT_SECS,
                                retry_count,
                                DEFAULT_MAX_RETRIES_PER_SKILL,
                            )
                        } else {
                            build_error_feedback(
                                &step_name,
                                error_msg,
                                retry_count,
                                DEFAULT_MAX_RETRIES_PER_SKILL,
                                &call.parameters,
                            )
                        };
                        messages.push(feedback);

                        // Check if we've reached max retries for this skill
                        let can_retry = retry_context.can_retry(&step_name);
                        if !can_retry {
                            let force_prompt = build_max_retries_exceeded_feedback(
                                &step_name,
                                DEFAULT_MAX_RETRIES_PER_SKILL,
                                error_msg,
                            );
                            messages.push(force_prompt);
                        }
                    }
                }
            }
            ReactInstruction::Batch(steps) => {
                let step_index = step_results.len();
                let step_name = format!("batch_{}_steps", steps.len());

                if let Err(result) = check_task_interruption(
                    task_id.as_deref(),
                    executor.get_workflow_callback(),
                    step_index,
                    &step_name,
                    None,
                )
                .await
                {
                    return result;
                }
                let batch_results = execute_batch_plan(executor, &steps).await;
                let has_failure = batch_results
                    .iter()
                    .any(|r| r.status == ExecutionStatus::Failure);
                if has_failure {
                    retry_context.record_failure(&step_name);
                } else {
                    retry_context.reset_consecutive_failures();
                }
                for result in &batch_results {
                    step_results.push(result.clone());
                }
                messages.push(ChatMessage::user(&format!(
                    "Batch execution completed. Results:\n{}",
                    format_step_results(&batch_results)
                )));
                if has_failure {
                    let failed_results: Vec<_> = batch_results
                        .iter()
                        .filter(|r| r.status == ExecutionStatus::Failure)
                        .collect();
                    let error_context = format!(
                        "Batch execution had {} failures:\n{}\n\nPlease decide how to proceed.",
                        failed_results.len(),
                        failed_results
                            .iter()
                            .map(|r| format!("- {}: {}", r.skill, r.output))
                            .collect::<Vec<_>>()
                            .join("\n")
                    );
                    messages.push(ChatMessage::user(&error_context));
                    continue;
                } else {
                    final_response = Some(format_step_results(&step_results));
                    break;
                }
            }
        }
    }
    if iteration >= executor.max_iterations {
        final_response = Some(t!("error.max_iterations_reached").to_string());
    }
    let final_response = final_response.unwrap_or_else(|| {
        if step_results.is_empty() {
            t!("skill.no_actions_executed").to_string()
        } else {
            format_step_results(&step_results)
        }
    });
    let raw_json = serde_json::json!({
        "mode": "react",
        "result": final_response,
        "steps": step_results.iter().map(|r| {
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
        display: final_response,
        raw_json,
    }
}
