//! OS time driver module
//!
//! This module provides functionality to get and set system time
//! (requires administrator privileges for setting).
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// A skill for getting system time.
#[derive(Debug)]
pub struct OsGetTimeDriver;
#[async_trait::async_trait]
impl Driver for OsGetTimeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "os_get_time";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get current system time and timezone";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to check the current date, time, and timezone";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "format".to_string(),
            param_type: "string".to_string(),
            description: "Output format: full, date, time, timestamp".to_string(),
            required: false,
            default: Some(json!("full")),
            example: Some(json!("date")),
            enum_values: Some(vec!["full".to_string(), "date".to_string(), "time".to_string(), "timestamp".to_string()]),
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_get_time"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Current time: 2024-01-15 14:30:45 (UTC+8)".to_string();
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
        debug!("Executing os_get_time driver");
        let format = parameters.get("format").and_then(|v| v.as_str()).unwrap_or("full");
        let now = chrono::Local::now();
        let tz = now.offset();
        debug!("Getting system time, format: {}", format);
        let result = match format {
            "date" => format!("Current date: {}", now.format("%Y-%m-%d")),
            "time" => format!("Current time: {}", now.format("%H:%M:%S")),
            "timestamp" => format!("Unix timestamp: {}", now.timestamp()),
            _ => format!("Current time: {} ({})", now.format("%Y-%m-%d %H:%M:%S"), tz),
        };
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
/// A skill for setting system time (requires admin).
#[derive(Debug)]
pub struct OsSetTimeDriver;
#[async_trait::async_trait]
impl Driver for OsSetTimeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "os_set_time";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Set system time (requires administrator privileges)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to adjust system time";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "datetime".to_string(),
            param_type: "string".to_string(),
            description: "New datetime in format 'YYYY-MM-DD HH:MM:SS'".to_string(),
            required: true,
            default: None,
            example: Some(json!("2024-01-15 12:00:00")),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_set_time",
            "parameters": {
                "datetime": "2024-01-15 10:00:00"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "System time updated successfully".to_string();
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
        debug!("Executing os_set_time driver");
        let datetime = parameters.get("datetime").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("datetime"))?;
        debug!("Setting system time to: {}", datetime);
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            exec_async("powershell", &["-Command", &format!("Set-Date -Date '{}'", datetime)], None)
                .await
                .map_err(|e| DriverError::execution(format!("Failed to set system time: {}", e)))?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            use crate::exec_async;
            exec_async("sudo", &["date", "-s", datetime], None)
                .await
                .map_err(|e| DriverError::execution(format!("Failed to set system time: {}", e)))?;
        }
        let result = "System time updated successfully".to_string();
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("datetime").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("datetime"))?;
        return Ok(());
    }
}
