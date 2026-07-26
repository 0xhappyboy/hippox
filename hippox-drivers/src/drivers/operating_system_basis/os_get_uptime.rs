//! OS get uptime driver
//!
//! This driver provides functionality to get system uptime information.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::System;
use tracing::{debug, info};
/// Driver for getting uptime information
#[derive(Debug)]
pub struct OsGetUptimeDriver;
#[async_trait::async_trait]
impl Driver for OsGetUptimeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_get_uptime"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get system uptime information"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to check how long the system has been running"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "human_readable".to_string(),
            param_type: "boolean".to_string(),
            description: "Return human-readable format (default: true)".to_string(),
            required: false,
            default: Some(json!(true)),
            example: Some(json!(false)),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_get_uptime"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "System uptime: 5 days, 3 hours, 22 minutes".to_string();
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
        debug!("Executing os_get_uptime driver");
        let human_readable = parameters.get("human_readable").and_then(|v| v.as_bool()).unwrap_or(true);
        let mut system = System::new();
        system.refresh_all();
        let uptime_secs = System::uptime();
        info!("Uptime retrieved: {} seconds", uptime_secs);
        if human_readable {
            debug!("Formatting uptime in human-readable format");
            let days = uptime_secs / 86400;
            let hours = (uptime_secs % 86400) / 3600;
            let minutes = (uptime_secs % 3600) / 60;
            let seconds = uptime_secs % 60;
            let mut parts = Vec::new();
            if days > 0 {
                parts.push(format!("{} days", days));
            }
            if hours > 0 {
                parts.push(format!("{} hours", hours));
            }
            if minutes > 0 {
                parts.push(format!("{} minutes", minutes));
            }
            if seconds > 0 && days == 0 && hours == 0 {
                parts.push(format!("{} seconds", seconds));
            }
            let result = format!("System uptime: {}", parts.join(", "));
            info!("Human-readable uptime: {}", result);
            return Ok(result);
        } else {
            let result = format!("System uptime: {} seconds", uptime_secs);
            info!("Raw uptime: {}", result);
            return Ok(result);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_get_uptime_metadata() {
        let driver = OsGetUptimeDriver;
        assert_eq!(driver.name(), "os_get_uptime");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
