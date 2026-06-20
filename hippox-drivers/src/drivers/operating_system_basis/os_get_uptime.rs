//! OS get uptime driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::System;
#[derive(Debug)]
pub struct OsGetUptimeDriver;
#[async_trait::async_trait]
impl Driver for OsGetUptimeDriver {
    fn name(&self) -> &str {
        "os_get_uptime"
    }
    fn description(&self) -> &str {
        "Get system uptime information"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to check how long the system has been running"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "human_readable".to_string(),
            param_type: "boolean".to_string(),
            description: "Return human-readable format (default: true)".to_string(),
            required: false,
            default: Some(json!(true)),
            example: Some(json!(false)),
            enum_values: None,
        }]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_get_uptime"
        })
    }
    fn example_output(&self) -> String {
        "System uptime: 5 days, 3 hours, 22 minutes".to_string()
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemBasis
    }
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let human_readable = parameters
            .get("human_readable")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let mut system = System::new();
        system.refresh_all();
        let uptime_secs = System::uptime();
        if human_readable {
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
            Ok(format!("System uptime: {}", parts.join(", ")))
        } else {
            Ok(format!("System uptime: {} seconds", uptime_secs))
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
