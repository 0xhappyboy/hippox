//! OS set time driver
//!
//! This driver provides functionality to set the system time (requires admin/root privileges).
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for setting the system time
#[derive(Debug)]
pub struct OsSetTimeDriver;
#[async_trait::async_trait]
impl Driver for OsSetTimeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_set_time"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set the system time (requires administrator/root privileges)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to set the system time. Requires admin/root privileges. Format: YYYY-MM-DD HH:MM:SS"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "datetime".to_string(),
            param_type: "string".to_string(),
            description: "Date and time in format: YYYY-MM-DD HH:MM:SS".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("2026-06-21 14:30:00".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_set_time",
            "parameters": {
                "datetime": "2026-06-21 14:30:00"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "System time set to 2026-06-21 14:30:00".to_string();
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
        debug!("Executing os_set_time driver");
        let datetime = parameters.get("datetime").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("datetime"))?;
        info!("Setting system time to: {}", datetime);
        #[cfg(target_os = "windows")]
        {
            debug!("Setting time on Windows");
            let output = Command::new("powershell").args(["-Command", &format!("Set-Date -Date '{}'", datetime)]).output();
            if let Ok(output) = output {
                if output.status.success() {
                    info!("System time set successfully on Windows");
                    return Ok(format!("System time set to {}", datetime));
                }
            }
            return Err(DriverError::execution("Failed to set system time. Requires administrator privileges."));
        }
        #[cfg(target_os = "linux")]
        {
            debug!("Setting time on Linux");
            let output = Command::new("sudo").args(["date", "-s", datetime]).output();
            if let Ok(output) = output {
                if output.status.success() {
                    info!("System time set successfully on Linux");
                    return Ok(format!("System time set to {}", datetime));
                }
            }
            return Err(DriverError::execution("Failed to set system time. Requires root privileges."));
        }
        #[cfg(target_os = "macos")]
        {
            debug!("Setting time on macOS");
            let output = Command::new("sudo").args(["date", &format!("{}", datetime)]).output();
            if let Ok(output) = output {
                if output.status.success() {
                    info!("System time set successfully on macOS");
                    return Ok(format!("System time set to {}", datetime));
                }
            }
            return Err(DriverError::execution("Failed to set system time. Requires root privileges."));
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            debug!("Setting time not supported on this platform");
            return Err(DriverError::execution("Setting system time is not supported on this platform"));
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
