//! OS set timezone driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
#[derive(Debug)]
pub struct OsSetTimezoneDriver;
#[async_trait::async_trait]
impl Driver for OsSetTimezoneDriver {
    fn name(&self) -> &str {
        "os_set_timezone"
    }
    fn description(&self) -> &str {
        "Set the system timezone (requires administrator/root privileges)"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to set the system timezone. Requires admin/root privileges. Example: Asia/Shanghai"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "timezone".to_string(),
            param_type: "string".to_string(),
            description: "Timezone name (e.g., Asia/Shanghai, America/New_York)".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Asia/Shanghai".to_string())),
            enum_values: None,
        }]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_set_timezone",
            "parameters": {
                "timezone": "Asia/Shanghai"
            }
        })
    }
    fn example_output(&self) -> String {
        "Timezone set to Asia/Shanghai".to_string()
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
        let timezone = parameters
            .get("timezone")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'timezone' parameter"))?;
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("powershell")
                .args([
                    "-Command",
                    &format!("Set-TimeZone -Id '{}'", timezone),
                ])
                .output();
            if let Ok(output) = output {
                if output.status.success() {
                    return Ok(format!("Timezone set to {}", timezone));
                }
            }
            return Err(anyhow::anyhow!("Failed to set timezone. Requires administrator privileges."));
        }
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("sudo")
                .args(["timedatectl", "set-timezone", timezone])
                .output();
            if let Ok(output) = output {
                if output.status.success() {
                    return Ok(format!("Timezone set to {}", timezone));
                }
            }
            return Err(anyhow::anyhow!("Failed to set timezone. Requires root privileges."));
        }
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("sudo")
                .args(["systemsetup", "-settimezone", timezone])
                .output();
            if let Ok(output) = output {
                if output.status.success() {
                    return Ok(format!("Timezone set to {}", timezone));
                }
            }
            return Err(anyhow::anyhow!("Failed to set timezone. Requires root privileges."));
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Err(anyhow::anyhow!("Setting timezone is not supported on this platform"))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_set_timezone_metadata() {
        let driver = OsSetTimezoneDriver;
        assert_eq!(driver.name(), "os_set_timezone");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}