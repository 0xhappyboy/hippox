//! Shared utility functions for workflow execution

use crate::t;
use super::types::{ExecutionStatus, StepResult};
use once_cell::sync::Lazy;
use regex::Regex;

/// Shared regex for variable resolution
pub static VARIABLE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{\{([^}]+)\}\}").unwrap()
});

/// Format step results for output
pub fn format_step_results(results: &[StepResult]) -> String {
    if results.is_empty() {
        return t!("skill.no_steps_executed").to_string();
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
        t!("skill.executed_steps", results.len()),
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