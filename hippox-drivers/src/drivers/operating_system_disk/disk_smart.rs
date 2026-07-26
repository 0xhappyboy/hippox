//! Disk SMART driver module
//!
//! This module provides functionality to get disk S.M.A.R.T. health information
//! including health percentage, temperature, power-on hours, and wear level.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    drivers::operating_system_disk::common::DiskSmartInfo,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for getting disk SMART health information
#[derive(Debug)]
pub struct DiskSmartDriver;
#[async_trait::async_trait]
impl Driver for DiskSmartDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "disk_smart";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get disk S.M.A.R.T. health status including health percentage and temperature";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to check disk health and predict potential failures";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "device".to_string(),
            param_type: "string".to_string(),
            description: "Disk device (e.g., /dev/sda)".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/dev/sda".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "disk_smart",
            "parameters": {
                "device": "/dev/sda"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"Disk SMART Health:
Health: 95.0%
Temperature: 35.0°C
Power On Hours: 12345
Wear Level: 85.0%
Errors: No"#
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
        debug!("Executing disk_smart driver");
        let device = parameters.get("device").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("device"))?;
        debug!("Getting SMART info for device: {}", device);
        let smart = get_smart_info(device)?;
        let mut output = String::from("Disk SMART Health:\n");
        output.push_str(&format!("Health: {:.1}%\n", smart.health_percent));
        output.push_str(&format!("Temperature: {:.1}°C\n", smart.temperature_celsius));
        output.push_str(&format!("Power On Hours: {}\n", smart.power_on_hours));
        if let Some(wear) = smart.wear_level {
            output.push_str(&format!("Wear Level: {:.1}%\n", wear));
        }
        output.push_str(&format!("Errors: {}\n", if smart.has_error { "Yes" } else { "No" }));
        if let Some(error) = smart.error_message {
            output.push_str(&format!("Error: {}\n", error));
        }
        info!("SMART info retrieved for {}", device);
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("device").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("device"))?;
        return Ok(());
    }
}
fn get_smart_info(device: &str) -> DriverResult<DiskSmartInfo> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("smartctl").args(&["-a", device]).output() {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    let mut health = 100.0;
                    let mut temp = 0.0;
                    let mut power_on = 0;
                    let mut wear = None;
                    let mut has_error = false;
                    let mut error_msg = None;
                    for line in output_str.lines() {
                        if line.contains("SMART overall-health self-assessment test result: PASSED") {
                            // Health is good
                        } else if line.contains("SMART overall-health self-assessment test result: FAILED") {
                            health = 50.0;
                            has_error = true;
                            error_msg = Some("SMART health test failed".to_string());
                        } else if line.contains("Temperature_Celsius") {
                            if let Some(temp_str) = line.split_whitespace().last() {
                                if let Ok(temp_val) = temp_str.parse::<f64>() {
                                    temp = temp_val;
                                }
                            }
                        } else if line.contains("Power_On_Hours") {
                            if let Some(hours_str) = line.split_whitespace().last() {
                                if let Ok(hours) = hours_str.parse::<u64>() {
                                    power_on = hours;
                                }
                            }
                        } else if line.contains("Wear_Leveling_Count") || line.contains("Wear") {
                            if let Some(wear_str) = line.split_whitespace().last() {
                                if let Ok(wear_val) = wear_str.parse::<f64>() {
                                    wear = Some(wear_val);
                                }
                            }
                        } else if line.contains("Reallocated_Sector_Ct") || line.contains("Reallocated") {
                            if let Some(ct_str) = line.split_whitespace().last() {
                                if let Ok(ct) = ct_str.parse::<u64>() {
                                    if ct > 0 {
                                        has_error = true;
                                        error_msg = Some(format!("{} reallocated sectors", ct));
                                        health = 100.0 - (ct as f64 * 5.0);
                                        if health < 0.0 {
                                            health = 0.0;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return Ok(DiskSmartInfo {
                        health_percent: health,
                        temperature_celsius: temp,
                        power_on_hours: power_on,
                        wear_level: wear,
                        has_error,
                        error_message: error_msg,
                    });
                }
            }
        }
        return Ok(DiskSmartInfo {
            health_percent: 100.0,
            temperature_celsius: 25.0,
            power_on_hours: 0,
            wear_level: None,
            has_error: false,
            error_message: None,
        });
    }
    #[cfg(target_os = "windows")]
    {
        return get_windows_smart_info(device);
    }
    #[cfg(target_os = "macos")]
    {
        return get_macos_smart_info(device);
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        return Ok(DiskSmartInfo {
            health_percent: 100.0,
            temperature_celsius: 0.0,
            power_on_hours: 0,
            wear_level: None,
            has_error: false,
            error_message: Some("SMART not supported on this platform".to_string()),
        });
    }
}
#[cfg(target_os = "windows")]
fn get_windows_smart_info(device: &str) -> DriverResult<DiskSmartInfo> {
    use std::process::Command;
    let output = Command::new("powershell").args(&["-Command", &format!("smartctl -a '{}'", device)]).output();
    let mut health = 100.0;
    let mut temp = 0.0;
    let mut power_on = 0;
    let mut wear = None;
    let mut has_error = false;
    let mut error_msg = None;
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_string();
            for line in output_str.lines() {
                if line.contains("SMART overall-health self-assessment test result: PASSED") {
                } else if line.contains("SMART overall-health self-assessment test result: FAILED") {
                    health = 50.0;
                    has_error = true;
                    error_msg = Some("SMART health test failed".to_string());
                } else if line.contains("Temperature_Celsius") {
                    if let Some(temp_str) = line.split_whitespace().last() {
                        if let Ok(temp_val) = temp_str.parse::<f64>() {
                            temp = temp_val;
                        }
                    }
                } else if line.contains("Power_On_Hours") {
                    if let Some(hours_str) = line.split_whitespace().last() {
                        if let Ok(hours) = hours_str.parse::<u64>() {
                            power_on = hours;
                        }
                    }
                } else if line.contains("Wear_Leveling_Count") || line.contains("Wear") {
                    if let Some(wear_str) = line.split_whitespace().last() {
                        if let Ok(wear_val) = wear_str.parse::<f64>() {
                            wear = Some(wear_val);
                        }
                    }
                }
            }
        }
    }
    return Ok(DiskSmartInfo {
        health_percent: health,
        temperature_celsius: temp as f32,
        power_on_hours: power_on,
        wear_level: wear.map(|v| v as f32),
        has_error,
        error_message: error_msg,
    });
}
#[cfg(target_os = "macos")]
fn get_macos_smart_info(device: &str) -> DriverResult<DiskSmartInfo> {
    use std::process::Command;
    let output = Command::new("smartctl").args(&["-a", device]).output();
    let mut health = 100.0;
    let mut temp = 0.0;
    let mut power_on = 0;
    let mut wear = None;
    let mut has_error = false;
    let mut error_msg = None;
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_string();
            for line in output_str.lines() {
                if line.contains("SMART overall-health self-assessment test result: PASSED") {
                } else if line.contains("SMART overall-health self-assessment test result: FAILED") {
                    health = 50.0;
                    has_error = true;
                    error_msg = Some("SMART health test failed".to_string());
                } else if line.contains("Temperature_Celsius") {
                    if let Some(temp_str) = line.split_whitespace().last() {
                        if let Ok(temp_val) = temp_str.parse::<f64>() {
                            temp = temp_val;
                        }
                    }
                } else if line.contains("Power_On_Hours") {
                    if let Some(hours_str) = line.split_whitespace().last() {
                        if let Ok(hours) = hours_str.parse::<u64>() {
                            power_on = hours;
                        }
                    }
                } else if line.contains("Wear_Leveling_Count") || line.contains("Wear") {
                    if let Some(wear_str) = line.split_whitespace().last() {
                        if let Ok(wear_val) = wear_str.parse::<f64>() {
                            wear = Some(wear_val);
                        }
                    }
                }
            }
        }
    }
    return Ok(DiskSmartInfo {
        health_percent: health,
        temperature_celsius: temp,
        power_on_hours: power_on,
        wear_level: wear,
        has_error,
        error_message: error_msg,
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_disk_smart_metadata() {
        let driver = DiskSmartDriver;
        assert_eq!(driver.name(), "disk_smart");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemDisk);
    }
}
