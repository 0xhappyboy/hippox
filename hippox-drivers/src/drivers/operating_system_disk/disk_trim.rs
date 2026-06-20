//! Disk TRIM driver

use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for checking and triggering disk TRIM
#[derive(Debug)]
pub struct DiskTrimDriver;

#[async_trait::async_trait]
impl Driver for DiskTrimDriver {
    fn name(&self) -> &str {
        "disk_trim"
    }

    fn description(&self) -> &str {
        "Check TRIM support and trigger TRIM operation on SSDs"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check SSD TRIM support or manually trigger TRIM"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "disk_trim",
            "parameters": {
                "device": "/dev/sda",
                "action": "check"
            }
        })
    }

    fn example_output(&self) -> String {
        r#"TRIM Support: Yes
TRIM Status: Active"#
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
            .unwrap_or("/dev/sda");

        let action = parameters
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("check");

        match action {
            "trigger" => trigger_trim(device),
            _ => check_trim(device),
        }
    }
}

fn check_trim(device: &str) -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        let device_name = device.trim_start_matches("/dev/");
        let discard_path = format!("/sys/block/{}/queue/discard_granularity", device_name);
        let rot_path = format!("/sys/block/{}/queue/rotational", device_name);

        let discard_granularity = std::fs::read_to_string(&discard_path)
            .map(|s| s.trim().parse::<u64>().unwrap_or(0))
            .unwrap_or(0);

        let is_rotational = std::fs::read_to_string(&rot_path)
            .map(|s| s.trim() == "1")
            .unwrap_or(true);

        let mut output = String::new();
        output.push_str(&format!(
            "TRIM Support: {}\n",
            if discard_granularity > 0 { "Yes" } else { "No" }
        ));
        output.push_str(&format!(
            "Drive Type: {}\n",
            if is_rotational { "HDD" } else { "SSD" }
        ));

        if discard_granularity > 0 {
            output.push_str(&format!(
                "TRIM Granularity: {} bytes\n",
                discard_granularity
            ));
        }

        Ok(output)
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok("TRIM support detection not available on this platform".to_string())
    }
}

fn trigger_trim(device: &str) -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        // Use fstrim command
        if let Ok(output) = std::process::Command::new("sudo")
            .args(&["fstrim", "-v", device])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    return Ok(format!("TRIM completed successfully:\n{}", output_str));
                }
            }
        }

        // Fallback: try fstrim without sudo
        if let Ok(output) = std::process::Command::new("fstrim")
            .args(&["-v", device])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    return Ok(format!("TRIM completed successfully:\n{}", output_str));
                }
            }
        }

        // Try via fstrim on mount point
        if let Ok(content) = std::fs::read_to_string("/proc/mounts") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[0] == device {
                    let mount_point = parts[1];
                    if let Ok(output) = std::process::Command::new("sudo")
                        .args(&["fstrim", "-v", mount_point])
                        .output()
                    {
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

        Err(anyhow::anyhow!(
            "Failed to trigger TRIM. Requires root privileges and fstrim command."
        ))
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err(anyhow::anyhow!(
            "TRIM triggering not supported on this platform"
        ))
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
