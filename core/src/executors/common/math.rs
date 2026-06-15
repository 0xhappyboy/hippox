use anyhow::Result;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use std::collections::HashMap;

/// Math common module
///
/// This module provides reusable mathematical utilities that can be used by math-related skills.
///
/// # Examples
///
/// ## Validate and parse numbers
///
/// ```rust
/// use crate::executors::utils::Math;
///
/// let num = Math::validate_number("3.14")?;
/// let integer = Math::validate_integer("42")?;
/// ```
///
/// ## Format numbers with precision
///
/// ```rust
/// use crate::executors::utils::Math;
///
/// let formatted = Math::format_number(3.1415926, 2);
/// assert_eq!(formatted, "3.14");
/// ```
///
/// ## Check if number is within range
///
/// ```rust
/// use crate::executors::utils::Math;
///
/// let is_in_range = Math::in_range(5.0, 0.0, 10.0);
/// assert!(is_in_range);
/// ```
///
/// ## Complete example in a skill
///
/// ```rust
/// use crate::executors::types::Skill;
/// use crate::executors::utils::Math;
///
/// async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
///     let value = parameters
///         .get("value")
///         .and_then(|v| v.as_str())
///         .ok_or_else(|| anyhow::anyhow!("Missing 'value' parameter"))?;
///     let num = Math::validate_number(value)?;
///     let result = num * 2.0;
///     let precision = parameters
///         .get("precision")
///         .and_then(|v| v.as_u64())
///         .unwrap_or(2);
///     Ok(Math::format_number(result, precision as usize))
/// }
/// ```
/// Validate numeric input
pub fn validate_number(value: &str) -> Result<f64> {
    value
        .parse::<f64>()
        .map_err(|_| anyhow::anyhow!("Invalid number: {}", value))
}

/// Validate integer input
pub fn validate_integer(value: &str) -> Result<i64> {
    value
        .parse::<i64>()
        .map_err(|_| anyhow::anyhow!("Invalid integer: {}", value))
}

/// Format number with appropriate precision
pub fn format_number(value: f64, precision: usize) -> String {
    format!("{:.1$}", value, precision)
}

/// Check if number is within range
pub fn in_range(value: f64, min: f64, max: f64) -> bool {
    value >= min && value <= max
}
