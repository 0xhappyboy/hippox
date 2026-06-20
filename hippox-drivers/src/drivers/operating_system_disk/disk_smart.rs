//! Disk SMART driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    drivers::operating_system_disk::common::DiskSmartInfo,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
/// Driver for getting disk SMART health information
#[derive(Debug)]
pub struct DiskSmartDriver;
#[async_trait::async_trait]
impl Driver for DiskSmartDriver {
    fn name(&self) -> &str {
        "disk_smart"
    }
    fn description(&self) -> &str {
        "Get disk S.M.A.R.T. health status including health percentage and temperature"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to check disk health and predict potential failures"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "device".to_string(),
            param_type: "string".to_string(),
            description: "Disk device (e.g., /dev/sda)".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/dev/sda".to_string())),
            enum_values: None,
        }]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "disk_smart",
            "parameters": {
                "device": "/dev/sda"
            }
        })
    }
    fn example_output(&self) -> String {
        r#"Disk SMART Health:
Health: 95.0%
Temperature: 35.0°C
Power On Hours: 12345
Wear Level: 85.0%
Errors: No"#
            .to_string()
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemDisk
    }
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let device = parameters
            .get("device")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: device"))?;
        let smart = get_smart_info(device)?;
        let mut output = String::from("Disk SMART Health:\n");
        output.push_str(&format!("Health: {:.1}%\n", smart.health_percent));
        output.push_str(&format!(
            "Temperature: {:.1}°C\n",
            smart.temperature_celsius
        ));
        output.push_str(&format!("Power On Hours: {}\n", smart.power_on_hours));
        if let Some(wear) = smart.wear_level {
            output.push_str(&format!("Wear Level: {:.1}%\n", wear));
        }
        output.push_str(&format!(
            "Errors: {}\n",
            if smart.has_error { "Yes" } else { "No" }
        ));
        if let Some(error) = smart.error_message {
            output.push_str(&format!("Error: {}\n", error));
        }
        Ok(output)
    }
}
fn get_smart_info(device: &str) -> Result<DiskSmartInfo> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("smartctl")
            .args(&["-a", device])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    let mut health = 100.0;
                    let mut temp = 0.0;
                    let mut power_on = 0;
                    let mut wear = None;
                    let mut has_error = false;
                    let mut error_msg = None;
                    for line in output_str.lines() {
                        if line.contains("SMART overall-health self-assessment test result: PASSED")
                        {
                        } else if line
                            .contains("SMART overall-health self-assessment test result: FAILED")
                        {
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
                        } else if line.contains("Reallocated_Sector_Ct")
                            || line.contains("Reallocated")
                        {
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
        Ok(DiskSmartInfo {
            health_percent: 100.0,
            temperature_celsius: 25.0,
            power_on_hours: 0,
            wear_level: None,
            has_error: false,
            error_message: None,
        })
    }
    #[cfg(target_os = "windows")]
    {
        get_windows_smart_info(device)
    }
    #[cfg(target_os = "macos")]
    {
        get_macos_smart_info(device)
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Ok(DiskSmartInfo {
            health_percent: 100.0,
            temperature_celsius: 0.0,
            power_on_hours: 0,
            wear_level: None,
            has_error: false,
            error_message: Some("SMART not supported on this platform".to_string()),
        })
    }
}
#[cfg(target_os = "windows")]
fn get_windows_smart_info(device: &str) -> Result<DiskSmartInfo> {
    use std::process::Command;
    let output = Command::new("powershell")
        .args(&["-Command", &format!("smartctl -a '{}'", device)])
        .output();
    let output_wmi = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-CimInstance -Namespace root/wmi -ClassName MSStorageDriver_FailurePredictStatus | Select-Object PredictFailure, Reason"
        ])
        .output();
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
                } else if line.contains("SMART overall-health self-assessment test result: FAILED")
                {
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
    if health == 100.0 && !has_error {
        if let Ok(output) = output_wmi {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout).to_string();
                for line in output_str.lines() {
                    if line.contains("PredictFailure") && line.contains("True") {
                        has_error = true;
                        error_msg = Some("SMART predicts drive failure".to_string());
                        health = 30.0;
                    }
                    if line.contains("Reason") && has_error {
                        if let Some(reason) = line.split(':').nth(1) {
                            error_msg = Some(reason.trim().to_string());
                        }
                    }
                }
            }
        }
    }
    Ok(DiskSmartInfo {
        health_percent: health,
        temperature_celsius: temp as f32,
        power_on_hours: power_on,
        wear_level: wear.map(|v| v as f32),
        has_error,
        error_message: error_msg,
    })
}
#[cfg(target_os = "macos")]
fn get_macos_smart_info(device: &str) -> Result<DiskSmartInfo> {
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
                } else if line.contains("SMART overall-health self-assessment test result: FAILED")
                {
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
    if health == 100.0 && !has_error {
        let output = Command::new("system_profiler")
            .args(&["SPStorageDataType"])
            .output();
        if let Ok(output) = output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout).to_string();
                for line in output_str.lines() {
                    if line.contains("S.M.A.R.T. Status") {
                        if line.contains("Verified") || line.contains("Passed") {
                        } else if line.contains("Failing") {
                            has_error = true;
                            error_msg = Some("S.M.A.R.T. status indicates failure".to_string());
                            health = 20.0;
                        } else if line.contains("Warning") {
                            has_error = true;
                            error_msg = Some("S.M.A.R.T. status warning".to_string());
                            health = 50.0;
                        }
                    }
                }
            }
        }
    }
    Ok(DiskSmartInfo {
        health_percent: health,
        temperature_celsius: temp,
        power_on_hours: power_on,
        wear_level: wear,
        has_error,
        error_message: error_msg,
    })
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
