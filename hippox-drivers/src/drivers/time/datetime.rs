//! DateTime driver module
//!
//! This module provides functionality to get current date and time,
//! perform timezone conversions, and get timestamps.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use chrono::{Duration, Local, Utc};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for datetime operations
#[derive(Debug)]
pub struct DateTimeDriver;
#[async_trait::async_trait]
impl Driver for DateTimeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "datetime";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get current date and time, or perform timezone conversions";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user asks for the current time, date, timestamp, or wants to convert between timezones";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "operation".to_string(),
                param_type: "string".to_string(),
                description: "Operation type: now, utc, timestamp, or convert".to_string(),
                required: false,
                default: Some(Value::String("now".to_string())),
                example: Some(Value::String("now".to_string())),
                enum_values: Some(vec!["now".to_string(), "utc".to_string(), "timestamp".to_string(), "convert".to_string()]),
            },
            DriverParameter {
                name: "timezone".to_string(),
                param_type: "string".to_string(),
                description: "Timezone like 'Asia/Shanghai', 'America/New_York', 'UTC' (for convert operation)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("Asia/Shanghai".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "format".to_string(),
                param_type: "string".to_string(),
                description: "Output format like '%Y-%m-%d %H:%M:%S'".to_string(),
                required: false,
                default: Some(Value::String("%Y-%m-%d %H:%M:%S".to_string())),
                example: Some(Value::String("%Y-%m-%d %H:%M:%S".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "datetime",
            "parameters": {
                "operation": "now"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "2024-01-15 14:30:25".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Time;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing datetime driver");
        let operation = parameters.get("operation").and_then(|v| v.as_str()).unwrap_or("now");
        let timezone = parameters.get("timezone").and_then(|v| v.as_str());
        let format_str = parameters.get("format").and_then(|v| v.as_str()).unwrap_or("%Y-%m-%d %H:%M:%S");
        debug!("Operation: {}, timezone: {:?}, format: {}", operation, timezone, format_str);
        let result = match operation {
            "utc" => {
                let now = Utc::now();
                now.format(format_str).to_string()
            }
            "timestamp" => {
                let now = Utc::now();
                now.timestamp().to_string()
            }
            "convert" => {
                if let Some(tz) = timezone {
                    convert_timezone(tz, format_str)?
                } else {
                    "Missing timezone for conversion".to_string()
                }
            }
            _ => {
                let now = Local::now();
                now.format(format_str).to_string()
            }
        };
        info!("Datetime operation completed: {}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
/// Converts UTC time to a specific timezone
///
/// # Arguments
/// * `timezone` - Timezone name (e.g., "Asia/Shanghai", "UTC+8")
/// * `format_str` - Output format string
///
/// # Returns
/// * `DriverResult<String>` - Formatted datetime in the target timezone
fn convert_timezone(timezone: &str, format_str: &str) -> DriverResult<String> {
    let offset = match timezone.to_lowercase().as_str() {
        "utc" | "gmt" => 0,
        "asia/shanghai" | "asia/beijing" | "cst" => 8,
        "asia/tokyo" | "jst" => 9,
        "america/new_york" | "est" => -5,
        "america/los_angeles" | "pst" => -8,
        "europe/london" | "bst" | "gmt" => 0,
        "europe/paris" | "cet" => 1,
        "asia/dubai" | "gst" => 4,
        "asia/singapore" | "sgt" => 8,
        "australia/sydney" | "aest" => 11,
        _ => {
            if let Some(offset_val) = parse_offset(timezone) {
                offset_val
            } else {
                return Err(DriverError::execution(format!("Unknown timezone: {}", timezone)));
            }
        }
    };
    let now = Utc::now();
    let dt = now + Duration::hours(offset.into());
    return Ok(dt.format(format_str).to_string());
}
/// Parses a timezone offset string like "UTC+8" or "GMT-5"
///
/// # Arguments
/// * `tz` - Timezone string
///
/// # Returns
/// * `Option<i32>` - Offset in hours, or None if parsing fails
fn parse_offset(tz: &str) -> Option<i32> {
    let tz_lower = tz.to_lowercase();
    if tz_lower.starts_with("utc") || tz_lower.starts_with("gmt") {
        let rest = &tz_lower[3..];
        if let Ok(offset) = rest.parse::<i32>() {
            return Some(offset);
        }
        if rest.starts_with('+') {
            if let Ok(offset) = rest[1..].parse::<i32>() {
                return Some(offset);
            }
        }
        if rest.starts_with('-') {
            if let Ok(offset) = rest[1..].parse::<i32>() {
                return Some(-offset);
            }
        }
    }
    return None;
}
