//! Shared utility functions for workflow execution

use std::sync::Arc;

use super::types::{ExecutionStatus, StepResult};
use crate::{ReactInstruction, StepInterruptionInfo, WorkflowCallback, WorkflowExecutionResult, WorkflowExecutor, t};
use hippox_drivers::DriverCall;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

/// Shared regex for variable resolution
pub static VARIABLE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{([^}]+)\}\}").unwrap());

/// Maximum length for output summary (200 characters)
pub const OUTPUT_SUMMARY_MAX_LEN: usize = 200;

/// Get output summary (truncated for display)
///
/// # Arguments
/// * `output` - The full output string
///
/// # Returns
/// Truncated string if longer than OUTPUT_SUMMARY_MAX_LEN, otherwise the original string
pub fn get_output_summary(output: &str) -> String {
    if output.len() <= OUTPUT_SUMMARY_MAX_LEN {
        output.to_string()
    } else {
        format!("{}...", &output[..OUTPUT_SUMMARY_MAX_LEN])
    }
}

/// Format step results for output
///
/// # Arguments
/// * `results` - Vector of StepResult
///
/// # Returns
/// Formatted string with step execution summary
pub fn format_step_results(results: &[StepResult]) -> String {
    if results.is_empty() {
        return t!("driver.no_steps_executed").to_string();
    }
    if results.len() == 1 {
        return results[0].output.clone();
    }
    let success_count = results
        .iter()
        .filter(|r| r.status == ExecutionStatus::Success)
        .count();
    let failure_count = results.len() - success_count;
    let mut output = format!(
        "{} (SUCCESS {} / FAILURE {}):\n\n",
        t!("driver.executed_steps", results.len()),
        success_count,
        failure_count
    );
    for (i, result) in results.iter().enumerate() {
        let marker = match result.status {
            ExecutionStatus::Success => "SUCCESS",
            ExecutionStatus::Failure => "FAILURE",
        };
        output.push_str(&format!("{} {}: {}\n", marker, i + 1, result.output));
    }
    output
}

/// Format duration in milliseconds to human-readable string
///
/// # Arguments
/// * `ms` - Duration in milliseconds
///
/// # Returns
/// Formatted string like "1.23s", "456ms", or "<1ms"
pub fn format_duration(ms: u64) -> String {
    if ms >= 1000 {
        format!("{:.2}s", ms as f64 / 1000.0)
    } else if ms > 0 {
        format!("{}ms", ms)
    } else {
        "<1ms".to_string()
    }
}

/// Format parameters for display (JSON string with limited length)
///
/// # Arguments
/// * `params` - Optional parameters map
///
/// # Returns
/// Formatted JSON string or "{}" if None
pub fn format_parameters(
    params: Option<&std::collections::HashMap<String, serde_json::Value>>,
) -> String {
    match params {
        Some(p) if !p.is_empty() => {
            let json_str = serde_json::to_string(p).unwrap_or_else(|_| "{}".to_string());
            if json_str.len() > 100 {
                format!("{}...", &json_str[..100])
            } else {
                json_str
            }
        }
        _ => "{}".to_string(),
    }
}

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
                let mut driver_calls = Vec::new();
                for step in steps {
                    let call: DriverCall = serde_json::from_value(step.clone())?;
                    driver_calls.push(call);
                }
                return Ok(ReactInstruction::Batch(driver_calls));
            }
        }
    }
    if let Ok(call) = serde_json::from_value(value) {
        return Ok(ReactInstruction::Single(call));
    }
    anyhow::bail!("Unable to parse LLM response: {}", response)
}

pub async fn check_task_interruption(
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
