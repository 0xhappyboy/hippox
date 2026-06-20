//! OS set time driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
#[derive(Debug)]
pub struct OsSetTimeDriver;
#[async_trait::async_trait]
impl Driver for OsSetTimeDriver {
    fn name(&self) -> &str {
        "os_set_time"
    }
    fn description(&self) -> &str {
        "Set the system time (requires administrator/root privileges)"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to set the system time. Requires admin/root privileges. Format: YYYY-MM-DD HH:MM:SS"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "datetime".to_string(),
            param_type: "string".to_string(),
            description: "Date and time in format: YYYY-MM-DD HH:MM:SS".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("2026-06-21 14:30:00".to_string())),
            enum_values: None,
        }]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_set_time",
            "parameters": {
                "datetime": "2026-06-21 14:30:00"
            }
        })
    }
    fn example_output(&self) -> String {
        "System time set to 2026-06-21 14:30:00".to_string()
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
        let datetime = parameters
            .get("datetime")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'datetime' parameter"))?;
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("powershell")
                .args([
                    "-Command",
                    &format!("Set-Date -Date '{}'", datetime),
                ])
                .output();
            if let Ok(output) = output {
                if output.status.success() {
                    return Ok(format!("System time set to {}", datetime));
                }
            }
            return Err(anyhow::anyhow!("Failed to set system time. Requires administrator privileges."));
        }
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("sudo")
                .args(["date", "-s", datetime])
                .output();
            if let Ok(output) = output {
                if output.status.success() {
                    return Ok(format!("System time set to {}", datetime));
                }
            }
            return Err(anyhow::anyhow!("Failed to set system time. Requires root privileges."));
        }
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("sudo")
                .args(["date", &format!("{}", datetime)])
                .output();
            if let Ok(output) = output {
                if output.status.success() {
                    return Ok(format!("System time set to {}", datetime));
                }
            }
            return Err(anyhow::anyhow!("Failed to set system time. Requires root privileges."));
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Err(anyhow::anyhow!("Setting system time is not supported on this platform"))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_set_time_metadata() {
        let driver = OsSetTimeDriver;
        assert_eq!(driver.name(), "os_set_time");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}