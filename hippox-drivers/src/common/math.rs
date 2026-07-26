//! Math common module
//!
//! This module provides reusable mathematical utilities that can be used by math-related skills.
//!
//! # Examples
//!
//! ## Validate and parse numbers
//!
//! ```rust
//! use crate::executors::utils::Math;
//!
//! let num = Math::validate_number("3.14")?;
//! let integer = Math::validate_integer("42")?;
//! ```
//!
//! ## Format numbers with precision
//!
//! ```rust
//! use crate::executors::utils::Math;
//!
//! let formatted = Math::format_number(3.1415926, 2);
//! assert_eq!(formatted, "3.14");
//! ```
//!
//! ## Check if number is within range
//!
//! ```rust
//! use crate::executors::utils::Math;
//!
//! let is_in_range = Math::in_range(5.0, 0.0, 10.0);
//! assert!(is_in_range);
//! ```
//!
//! ## Complete example in a skill
//!
//! ```rust
//! use crate::executors::types::Skill;
//! use crate::executors::utils::Math;
//!
//! async fn execute(&self, parameters: &HashMap<String, Value>) -> DriverResult<String> {
//!     let value = parameters
//!         .get("value")
//!         .and_then(|v| v.as_str())
//!         .ok_or_else(|| DriverError::missing_parameter("value"))?;
//!     let num = Math::validate_number(value)?;
//!     let result = num * 2.0;
//!     let precision = parameters
//!         .get("precision")
//!         .and_then(|v| v.as_u64())
//!         .unwrap_or(2);
//!     Ok(Math::format_number(result, precision as usize))
//! }
//! ```
use crate::DriverError;
use crate::result::DriverResult;
use tracing::{debug, info, warn};
/// Validate numeric input
pub fn validate_number(value: &str) -> DriverResult<f64> {
    debug!("Validating number: {}", value);
    match value.parse::<f64>() {
        Ok(num) => {
            info!("Number validated: {}", num);
            return Ok(num);
        }
        Err(_) => {
            let err_msg = format!("Invalid number: {}", value);
            warn!("{}", err_msg);
            return Err(DriverError::validation("value", err_msg));
        }
    }
}
/// Validate integer input
pub fn validate_integer(value: &str) -> DriverResult<i64> {
    debug!("Validating integer: {}", value);
    match value.parse::<i64>() {
        Ok(num) => {
            info!("Integer validated: {}", num);
            return Ok(num);
        }
        Err(_) => {
            let err_msg = format!("Invalid integer: {}", value);
            warn!("{}", err_msg);
            return Err(DriverError::validation("value", err_msg));
        }
    }
}
/// Format number with appropriate precision
pub fn format_number(value: f64, precision: usize) -> String {
    debug!("Formatting number: {} with precision {}", value, precision);
    let result = format!("{:.1$}", value, precision);
    info!("Number formatted: {}", result);
    return result;
}
/// Check if number is within range
pub fn in_range(value: f64, min: f64, max: f64) -> bool {
    debug!("Checking if {} is in range [{}, {}]", value, min, max);
    let result = value >= min && value <= max;
    info!("Number in range: {}", result);
    return result;
}
