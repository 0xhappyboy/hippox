//! OS get time driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use chrono::Local;
use serde_json::{Value, json};
use std::collections::HashMap;
#[derive(Debug)]
pub struct OsGetTimeDriver;
#[async_trait::async_trait]
impl Driver for OsGetTimeDriver {
    fn name(&self) -> &str {
        "os_get_time"
    }
    fn description(&self) -> &str {
        "Get current system time, date, and timezone information"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to get the current time, date, and timezone"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "format".to_string(),
            param_type: "string".to_string(),
            description: "Output format: full, date, time, timestamp, iso".to_string(),
            required: false,
            default: Some(Value::String("full".to_string())),
            example: Some(Value::String("iso".to_string())),
            enum_values: Some(vec![
                "full".to_string(),
                "date".to_string(),
                "time".to_string(),
                "timestamp".to_string(),
                "iso".to_string(),
            ]),
        }]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_get_time",
            "parameters": {
                "format": "full"
            }
        })
    }
    fn example_output(&self) -> String {
        "Current time: 2026-06-21 14:30:45 (UTC+8)\nTimestamp: 1718965845\nTimezone: Asia/Shanghai"
            .to_string()
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
        let format = parameters
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("full");
        let now = Local::now();
        let timestamp = now.timestamp();
        let tz = now.offset().to_owned();
        match format {
            "date" => Ok(now.format("%Y-%m-%d").to_string()),
            "time" => Ok(now.format("%H:%M:%S").to_string()),
            "timestamp" => Ok(timestamp.to_string()),
            "iso" => Ok(now.to_rfc3339()),
            _ => Ok(format!(
                "Current time: {}\nTimestamp: {}\nTimezone: {}",
                now.format("%Y-%m-%d %H:%M:%S"),
                timestamp,
                tz
            )),
        }
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
