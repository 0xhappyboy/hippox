//! Retry, timeout, and error handling utilities for skill execution
//!
//! This module provides unified skill execution capabilities that can be
//! used across all workflow modes (ReAct, Batch, Chain, PlanAndExecute).

use crate::prompts::{
    build_consecutive_failures_prompt, build_error_feedback_prompt,
    build_max_retries_exceeded_prompt, build_timeout_feedback_prompt,
};
use hippox_atomic_skills::{Executor, SkillCall, SkillCallback, SkillContext};
use langhub::types::ChatMessage;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use super::core::WorkflowExecutor;
use super::types::*;

/// Default timeout for individual skill execution (in seconds)
pub const DEFAULT_SKILL_TIMEOUT_SECS: u64 = 60;

/// Maximum number of retries for a single skill
pub const DEFAULT_MAX_RETRIES_PER_SKILL: usize = 3;

/// Maximum number of consecutive failures before forcing a decision
pub const DEFAULT_MAX_CONSECUTIVE_FAILURES: usize = 5;

/// Manages retry state for skills in the current workflow
#[derive(Debug, Clone)]
pub struct RetryContext {
    /// Maximum retries per skill
    max_retries: usize,
    /// Current retry count per skill name
    retry_counts: HashMap<String, usize>,
    /// Set of skills that have permanently failed
    failed_skills: HashSet<String>,
    /// Consecutive failure count across all skills
    consecutive_failures: usize,
    /// Maximum consecutive failures allowed
    max_consecutive_failures: usize,
}

impl RetryContext {
    pub fn new(max_retries: usize, max_consecutive_failures: usize) -> Self {
        Self {
            max_retries,
            retry_counts: HashMap::new(),
            failed_skills: HashSet::new(),
            consecutive_failures: 0,
            max_consecutive_failures,
        }
    }

    /// Check if a skill can be retried (consumes one retry quota)
    pub fn can_retry(&mut self, skill_name: &str) -> bool {
        if self.failed_skills.contains(skill_name) {
            return false;
        }
        let count = self.retry_counts.entry(skill_name.to_string()).or_insert(0);
        let result = *count < self.max_retries;
        if result {
            *count += 1;
        } else {
            self.failed_skills.insert(skill_name.to_string());
        }
        result
    }

    /// Get current retry count for a skill (number of attempts already made)
    pub fn get_retry_count(&self, skill_name: &str) -> usize {
        self.retry_counts.get(skill_name).copied().unwrap_or(0)
    }

    /// Check if a skill can be retried without consuming quota (just peek)
    pub fn can_retry_peek(&self, skill_name: &str) -> bool {
        if self.failed_skills.contains(skill_name) {
            return false;
        }
        let count = self.retry_counts.get(skill_name).copied().unwrap_or(0);
        count < self.max_retries
    }

    /// Record a failure and increment consecutive failure counter
    pub fn record_failure(&mut self, _skill_name: &str) {
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

    /// Check if a skill has permanently failed
    pub fn is_skill_permanently_failed(&self, skill_name: &str) -> bool {
        self.failed_skills.contains(skill_name)
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

/// Result of executing a skill with timeout protection
#[derive(Debug, Clone)]
pub enum SkillExecutionResult {
    /// Skill executed successfully with output
    Success(String),
    /// Skill failed with error message
    Failure(String),
    /// Skill timed out
    Timeout(String),
}

impl SkillExecutionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, SkillExecutionResult::Success(_))
    }

    pub fn is_timeout(&self) -> bool {
        matches!(self, SkillExecutionResult::Timeout(_))
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, SkillExecutionResult::Failure(_))
    }

    pub fn into_output(self) -> Option<String> {
        match self {
            SkillExecutionResult::Success(output) => Some(output),
            _ => None,
        }
    }

    pub fn into_error(self) -> Option<String> {
        match self {
            SkillExecutionResult::Failure(msg) => Some(msg),
            SkillExecutionResult::Timeout(msg) => Some(msg),
            _ => None,
        }
    }
}

/// Execute a skill with timeout protection
pub async fn execute_skill_with_timeout(
    executor: &Executor,
    call: &SkillCall,
    skill_callback: Option<Arc<dyn SkillCallback>>,
    skill_context: Option<&SkillContext>,
    timeout_secs: u64,
) -> SkillExecutionResult {
    let call_clone = call.clone();
    match tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        executor.execute(&call_clone, skill_callback.as_deref(), skill_context),
    )
    .await
    {
        Ok(Ok(output)) => SkillExecutionResult::Success(output),
        Ok(Err(e)) => SkillExecutionResult::Failure(e.to_string()),
        Err(_) => SkillExecutionResult::Timeout(format!(
            "Skill '{}' timed out after {} seconds",
            call.action, timeout_secs
        )),
    }
}

/// Build timeout feedback for LLM
pub fn build_timeout_feedback(
    skill_name: &str,
    timeout_secs: u64,
    retry_count: usize,
    max_retries: usize,
) -> ChatMessage {
    let prompt = build_timeout_feedback_prompt(skill_name, timeout_secs, retry_count, max_retries);
    ChatMessage::user(&prompt)
}

/// Build error feedback for LLM
pub fn build_error_feedback(
    skill_name: &str,
    error_msg: &str,
    retry_count: usize,
    max_retries: usize,
    parameters: &HashMap<String, Value>,
) -> ChatMessage {
    let prompt = build_error_feedback_prompt(
        skill_name,
        error_msg,
        retry_count,
        max_retries,
        &serde_json::to_value(parameters).unwrap_or_default(),
    );
    ChatMessage::user(&prompt)
}

/// Build max retries exceeded feedback for LLM
pub fn build_max_retries_exceeded_feedback(
    skill_name: &str,
    max_retries: usize,
    error_msg: &str,
) -> ChatMessage {
    let prompt = build_max_retries_exceeded_prompt(skill_name, max_retries, error_msg);
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
pub fn create_step_result(call: &SkillCall, result: &SkillExecutionResult) -> StepResult {
    let (output, status) = match result {
        SkillExecutionResult::Success(output) => (output.clone(), ExecutionStatus::Success),
        SkillExecutionResult::Failure(msg) => (msg.clone(), ExecutionStatus::Failure),
        SkillExecutionResult::Timeout(msg) => (msg.clone(), ExecutionStatus::Failure),
    };
    StepResult {
        skill: call.action.clone(),
        parameters: call.parameters.clone(),
        output,
        status,
    }
}

/// Create a failed step result from an error
pub fn create_failed_step_result(call: &SkillCall, error: &str) -> StepResult {
    StepResult {
        skill: call.action.clone(),
        parameters: call.parameters.clone(),
        output: error.to_string(),
        status: ExecutionStatus::Failure,
    }
}

/// Create a success step result
pub fn create_success_step_result(call: &SkillCall, output: &str) -> StepResult {
    StepResult {
        skill: call.action.clone(),
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
pub fn result_is_timeout(result: &SkillExecutionResult) -> bool {
    matches!(result, SkillExecutionResult::Timeout(_))
}

/// Get error message from result
pub fn get_error_message(result: &SkillExecutionResult) -> Option<&str> {
    match result {
        SkillExecutionResult::Failure(msg) => Some(msg),
        SkillExecutionResult::Timeout(msg) => Some(msg),
        SkillExecutionResult::Success(_) => None,
    }
}

/// Get timeout value (from executor or default)
pub fn get_timeout_secs(executor: &WorkflowExecutor) -> u64 {
    DEFAULT_SKILL_TIMEOUT_SECS
}
