//! OS set timezone driver
//!
//! This driver provides functionality to set the system timezone (requires admin/root privileges).
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for setting the system timezone
#[derive(Debug)]
pub struct OsSetTimezoneDriver;
#[async_trait::async_trait]
impl Driver for OsSetTimezoneDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_set_timezone"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set the system timezone (requires administrator/root privileges)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to set the system timezone. Requires admin/root privileges. Example: Asia/Shanghai"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "timezone".to_string(),
            param_type: "string".to_string(),
            description: "Timezone name (e.g., Asia/Shanghai, America/New_York)".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Asia/Shanghai".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_set_timezone",
            "parameters": {
                "timezone": "Asia/Shanghai"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Timezone set to Asia/Shanghai".to_string();
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
        debug!("Executing os_set_timezone driver");
        let timezone = parameters.get("timezone").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("timezone"))?;
        info!("Setting system timezone to: {}", timezone);
        #[cfg(target_os = "windows")]
        {
            debug!("Setting timezone on Windows");
            let output = Command::new("powershell").args(["-Command", &format!("Set-TimeZone -Id '{}'", timezone)]).output();
            if let Ok(output) = output {
                if output.status.success() {
                    info!("Timezone set successfully on Windows");
                    return Ok(format!("Timezone set to {}", timezone));
                }
            }
            return Err(DriverError::execution("Failed to set timezone. Requires administrator privileges."));
        }
        #[cfg(target_os = "linux")]
        {
            debug!("Setting timezone on Linux");
            let output = Command::new("sudo").args(["timedatectl", "set-timezone", timezone]).output();
            if let Ok(output) = output {
                if output.status.success() {
                    info!("Timezone set successfully on Linux");
                    return Ok(format!("Timezone set to {}", timezone));
                }
            }
            return Err(DriverError::execution("Failed to set timezone. Requires root privileges."));
        }
        #[cfg(target_os = "macos")]
        {
            debug!("Setting timezone on macOS");
            let output = Command::new("sudo").args(["systemsetup", "-settimezone", timezone]).output();
            if let Ok(output) = output {
                if output.status.success() {
                    info!("Timezone set successfully on macOS");
                    return Ok(format!("Timezone set to {}", timezone));
                }
            }
            return Err(DriverError::execution("Failed to set timezone. Requires root privileges."));
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            debug!("Setting timezone not supported on this platform");
            return Err(DriverError::execution("Setting timezone is not supported on this platform"));
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
