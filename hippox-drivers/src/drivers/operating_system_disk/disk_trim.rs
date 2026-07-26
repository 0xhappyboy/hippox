//! Disk TRIM driver module
//!
//! This module provides functionality to check TRIM support and trigger
//! TRIM operations on SSDs.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use tracing::{debug, info};
/// Driver for checking and triggering disk TRIM
#[derive(Debug)]
pub struct DiskTrimDriver;
#[async_trait::async_trait]
impl Driver for DiskTrimDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "disk_trim";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Check TRIM support and trigger TRIM operation on SSDs";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to check SSD TRIM support or manually trigger TRIM";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "device".to_string(),
                param_type: "string".to_string(),
                description: "Disk device (e.g., /dev/sda)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/dev/sda".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "action".to_string(),
                param_type: "string".to_string(),
                description: "Action: check (default) or trigger".to_string(),
                required: false,
                default: Some(Value::String("check".to_string())),
                example: Some(Value::String("check".to_string())),
                enum_values: Some(vec!["check".to_string(), "trigger".to_string()]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "disk_trim",
            "parameters": {
                "device": "/dev/sda",
                "action": "check"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"TRIM Support: Yes
TRIM Status: Active"#
            .to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemDisk;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing disk_trim driver");
        let device = parameters.get("device").and_then(|v| v.as_str()).unwrap_or("/dev/sda");
        let action = parameters.get("action").and_then(|v| v.as_str()).unwrap_or("check");
        debug!("TRIM action: {}, device: {}", action, device);
        let result = match action {
            "trigger" => trigger_trim(device)?,
            _ => check_trim(device)?,
        };
        info!("TRIM operation completed: {}", action);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
fn check_trim(device: &str) -> DriverResult<String> {
    #[cfg(target_os = "linux")]
    {
        let device_name = device.trim_start_matches("/dev/");
        let discard_path = format!("/sys/block/{}/queue/discard_granularity", device_name);
        let rot_path = format!("/sys/block/{}/queue/rotational", device_name);
        let discard_granularity = fs::read_to_string(&discard_path).map(|s| s.trim().parse::<u64>().unwrap_or(0)).unwrap_or(0);
        let is_rotational = fs::read_to_string(&rot_path).map(|s| s.trim() == "1").unwrap_or(true);
        let mut output = String::new();
        output.push_str(&format!("TRIM Support: {}\n", if discard_granularity > 0 { "Yes" } else { "No" }));
        output.push_str(&format!("Drive Type: {}\n", if is_rotational { "HDD" } else { "SSD" }));
        if discard_granularity > 0 {
            output.push_str(&format!("TRIM Granularity: {} bytes\n", discard_granularity));
        }
        return Ok(output);
    }
    #[cfg(not(target_os = "linux"))]
    {
        return Ok("TRIM support detection not available on this platform".to_string());
    }
}
fn trigger_trim(device: &str) -> DriverResult<String> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("sudo").args(&["fstrim", "-v", device]).output() {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    return Ok(format!("TRIM completed successfully:\n{}", output_str));
                }
            }
        }
        if let Ok(output) = Command::new("fstrim").args(&["-v", device]).output() {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    return Ok(format!("TRIM completed successfully:\n{}", output_str));
                }
            }
        }
        if let Ok(content) = fs::read_to_string("/proc/mounts") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[0] == device {
                    let mount_point = parts[1];
                    if let Ok(output) = Command::new("sudo").args(&["fstrim", "-v", mount_point]).output() {
                        if output.status.success() {
                            if let Ok(output_str) = String::from_utf8(output.stdout) {
                                return Ok(format!("TRIM completed successfully:\n{}", output_str));
                            }
                        }
                    }
                    break;
                }
            }
        }
        return Err(DriverError::execution("Failed to trigger TRIM. Requires root privileges and fstrim command.".to_string()));
    }
    #[cfg(not(target_os = "linux"))]
    {
        return Err(DriverError::execution("TRIM triggering not supported on this platform".to_string()));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_disk_trim_metadata() {
        let driver = DiskTrimDriver;
        assert_eq!(driver.name(), "disk_trim");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemDisk);
    }
}
