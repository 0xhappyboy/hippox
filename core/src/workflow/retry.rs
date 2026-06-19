//! Retry, timeout, and error handling utilities for driver execution
//!
//! This module provides unified driver execution capabilities that can be
//! used across all workflow modes (ReAct, Batch, Chain, PlanAndExecute).

use crate::prompts::{
    build_consecutive_failures_prompt, build_error_feedback_prompt,
    build_max_retries_exceeded_prompt, build_timeout_feedback_prompt,
};
use hippox_drivers::{Executor, DriverCall, DriverCallback, DriverContext};
use langhub::types::ChatMessage;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use super::core::WorkflowExecutor;
use super::types::*;

/// Default timeout for individual driver execution (in seconds)
pub const DEFAULT_SKILL_TIMEOUT_SECS: u64 = 60;

/// Maximum number of retries for a single driver
pub const DEFAULT_MAX_RETRIES_PER_SKILL: usize = 3;

/// Maximum number of consecutive failures before forcing a decision
pub const DEFAULT_MAX_CONSECUTIVE_FAILURES: usize = 5;

/// Manages retry state for drivers in the current workflow
#[derive(Debug, Clone)]
pub struct RetryContext {
    /// Maximum retries per driver
    max_retries: usize,
    /// Current retry count per driver name
    retry_counts: HashMap<String, usize>,
    /// Set of drivers that have permanently failed
    failed_drivers: HashSet<String>,
    /// Consecutive failure count across all drivers
    consecutive_failures: usize,
    /// Maximum consecutive failures allowed
    max_consecutive_failures: usize,
}

impl RetryContext {
    pub fn new(max_retries: usize, max_consecutive_failures: usize) -> Self {
        Self {
            max_retries,
            retry_counts: HashMap::new(),
            failed_drivers: HashSet::new(),
            consecutive_failures: 0,
            max_consecutive_failures,
        }
    }

    /// Check if a driver can be retried (consumes one retry quota)
    pub fn can_retry(&mut self, driver_name: &str) -> bool {
        if self.failed_drivers.contains(driver_name) {
            return false;
        }
        let count = self.retry_counts.entry(driver_name.to_string()).or_insert(0);
        let result = *count < self.max_retries;
        if result {
            *count += 1;
        } else {
            self.failed_drivers.insert(driver_name.to_string());
        }
        result
    }

    /// Get current retry count for a driver (number of attempts already made)
    pub fn get_retry_count(&self, driver_name: &str) -> usize {
        self.retry_counts.get(driver_name).copied().unwrap_or(0)
    }

    /// Check if a driver can be retried without consuming quota (just peek)
    pub fn can_retry_peek(&self, driver_name: &str) -> bool {
        if self.failed_drivers.contains(driver_name) {
            return false;
        }
        let count = self.retry_counts.get(driver_name).copied().unwrap_or(0);
        count < self.max_retries
    }

    /// Record a failure and increment consecutive failure counter
    pub fn record_failure(&mut self, _driver_name: &str) {
        self.consecutive_failures += 1;
    }

    /// Reset consecutive failure counter (called on success)
    pub fn reset_consecutive_failures(&mut self) {
        self.consecutive_failures = 0;
    }

    /// Check if consecutive failure threshold has been reached
    pub fn has_exceeded_consecutive_failures(&self) -> bool {
        self.consecutive_failures >= self.max_consecutive_failures
    }

    /// Check if a driver has permanently failed
    pub fn is_driver_permanently_failed(&self, driver_name: &str) -> bool {
        self.failed_drivers.contains(driver_name)
    }

    /// Get consecutive failures count
    pub fn consecutive_failures(&self) -> usize {
        self.consecutive_failures
    }

    /// Get max consecutive failures limit
    pub fn max_consecutive_failures(&self) -> usize {
        self.max_consecutive_failures
    }
}

/// Result of executing a driver with timeout protection
#[derive(Debug, Clone)]
pub enum DriverExecutionResult {
    /// Driver executed successfully with output
    Success(String),
    /// Driver failed with error message
    Failure(String),
    /// Driver timed out
    Timeout(String),
}

impl DriverExecutionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, DriverExecutionResult::Success(_))
    }

    pub fn is_timeout(&self) -> bool {
        matches!(self, DriverExecutionResult::Timeout(_))
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, DriverExecutionResult::Failure(_))
    }

    pub fn into_output(self) -> Option<String> {
        match self {
            DriverExecutionResult::Success(output) => Some(output),
            _ => None,
        }
    }

    pub fn into_error(self) -> Option<String> {
        match self {
            DriverExecutionResult::Failure(msg) => Some(msg),
            DriverExecutionResult::Timeout(msg) => Some(msg),
            _ => None,
        }
    }
}

/// Execute a driver with timeout protection
pub async fn execute_driver_with_timeout(
    executor: &Executor,
    call: &DriverCall,
    driver_callback: Option<Arc<dyn DriverCallback>>,
    driver_context: Option<&DriverContext>,
    timeout_secs: u64,
) -> DriverExecutionResult {
    let call_clone = call.clone();
    match tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        executor.execute(&call_clone, driver_callback.as_deref(), driver_context),
    )
    .await
    {
        Ok(Ok(output)) => DriverExecutionResult::Success(output),
        Ok(Err(e)) => DriverExecutionResult::Failure(e.to_string()),
        Err(_) => DriverExecutionResult::Timeout(format!(
            "Driver '{}' timed out after {} seconds",
            call.action, timeout_secs
        )),
    }
}

/// Build timeout feedback for LLM
pub fn build_timeout_feedback(
    driver_name: &str,
    timeout_secs: u64,
    retry_count: usize,
    max_retries: usize,
) -> ChatMessage {
    let prompt = build_timeout_feedback_prompt(driver_name, timeout_secs, retry_count, max_retries);
    ChatMessage::user(&prompt)
}

/// Build error feedback for LLM
pub fn build_error_feedback(
    driver_name: &str,
    error_msg: &str,
    retry_count: usize,
    max_retries: usize,
    parameters: &HashMap<String, Value>,
) -> ChatMessage {
    let prompt = build_error_feedback_prompt(
        driver_name,
        error_msg,
        retry_count,
        max_retries,
        &serde_json::to_value(parameters).unwrap_or_default(),
    );
    ChatMessage::user(&prompt)
}

/// Build max retries exceeded feedback for LLM
pub fn build_max_retries_exceeded_feedback(
    driver_name: &str,
    max_retries: usize,
    error_msg: &str,
) -> ChatMessage {
    let prompt = build_max_retries_exceeded_prompt(driver_name, max_retries, error_msg);
    ChatMessage::user(&prompt)
}

/// Build consecutive failures warning for LLM
pub fn build_consecutive_failures_feedback(
    consecutive_failures: usize,
    max_consecutive_failures: usize,
) -> ChatMessage {
    let prompt = build_consecutive_failures_prompt(consecutive_failures, max_consecutive_failures);
    ChatMessage::system(&prompt)
}

/// Create a step result from execution result
pub fn create_step_result(call: &DriverCall, result: &DriverExecutionResult) -> StepResult {
    let (output, status) = match result {
        DriverExecutionResult::Success(output) => (output.clone(), ExecutionStatus::Success),
        DriverExecutionResult::Failure(msg) => (msg.clone(), ExecutionStatus::Failure),
        DriverExecutionResult::Timeout(msg) => (msg.clone(), ExecutionStatus::Failure),
    };
    StepResult {
        driver: call.action.clone(),
        parameters: call.parameters.clone(),
        output,
        status,
    }
}

/// Create a failed step result from an error
pub fn create_failed_step_result(call: &DriverCall, error: &str) -> StepResult {
    StepResult {
        driver: call.action.clone(),
        parameters: call.parameters.clone(),
        output: error.to_string(),
        status: ExecutionStatus::Failure,
    }
}

/// Create a success step result
pub fn create_success_step_result(call: &DriverCall, output: &str) -> StepResult {
    StepResult {
        driver: call.action.clone(),
        parameters: call.parameters.clone(),
        output: output.to_string(),
        status: ExecutionStatus::Success,
    }
}

/// Check if an error message indicates a timeout
pub fn is_timeout_error(error_msg: &str) -> bool {
    error_msg.contains("timed out") || error_msg.contains("timeout")
}

/// Check if a result is a timeout
pub fn result_is_timeout(result: &DriverExecutionResult) -> bool {
    matches!(result, DriverExecutionResult::Timeout(_))
}

/// Get error message from result
pub fn get_error_message(result: &DriverExecutionResult) -> Option<&str> {
    match result {
        DriverExecutionResult::Failure(msg) => Some(msg),
        DriverExecutionResult::Timeout(msg) => Some(msg),
        DriverExecutionResult::Success(_) => None,
    }
}

/// Get timeout value (from executor or default)
pub fn get_timeout_secs(executor: &WorkflowExecutor) -> u64 {
    DEFAULT_SKILL_TIMEOUT_SECS
}
