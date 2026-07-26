//! OS get time driver
//!
//! This driver provides functionality to get current system time, date, and timezone information.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use chrono::Local;
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting time information
#[derive(Debug)]
pub struct OsGetTimeDriver;
#[async_trait::async_trait]
impl Driver for OsGetTimeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_get_time"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get current system time, date, and timezone information"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the current time, date, and timezone"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "format".to_string(),
            param_type: "string".to_string(),
            description: "Output format: full, date, time, timestamp, iso".to_string(),
            required: false,
            default: Some(Value::String("full".to_string())),
            example: Some(Value::String("iso".to_string())),
            enum_values: Some(vec!["full".to_string(), "date".to_string(), "time".to_string(), "timestamp".to_string(), "iso".to_string()]),
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_get_time",
            "parameters": {
                "format": "full"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Current time: 2026-06-21 14:30:45 (UTC+8)\nTimestamp: 1718965845\nTimezone: Asia/Shanghai".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemBasis;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing os_get_time driver");
        let format = parameters.get("format").and_then(|v| v.as_str()).unwrap_or("full");
        let now = Local::now();
        let timestamp = now.timestamp();
        let tz = now.offset().to_owned();
        let result = match format {
            "date" => {
                debug!("Getting date only");
                now.format("%Y-%m-%d").to_string()
            }
            "time" => {
                debug!("Getting time only");
                now.format("%H:%M:%S").to_string()
            }
            "timestamp" => {
                debug!("Getting timestamp");
                timestamp.to_string()
            }
            "iso" => {
                debug!("Getting ISO format");
                now.to_rfc3339()
            }
            _ => {
                debug!("Getting full time information");
                format!("Current time: {}\nTimestamp: {}\nTimezone: {}", now.format("%Y-%m-%d %H:%M:%S"), timestamp, tz)
            }
        };
        info!("Time information retrieved successfully");
        return Ok(result);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_get_time_metadata() {
        let driver = OsGetTimeDriver;
        assert_eq!(driver.name(), "os_get_time");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
