//! Disk IOPS driver module
//!
//! This module provides functionality to get disk IOPS (Input/Output Operations
//! Per Second) for read and write operations.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    drivers::operating_system_disk::common::DiskIopsInfo,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;
use tracing::{debug, info};
/// Driver for getting disk IOPS (Input/Output Operations Per Second)
#[derive(Debug)]
pub struct DiskIopsDriver;
#[async_trait::async_trait]
impl Driver for DiskIopsDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "disk_iops";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get disk IOPS (Input/Output Operations Per Second) for read and write";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to measure disk performance in IOPS";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "device".to_string(),
                param_type: "string".to_string(),
                description: "Disk device (e.g., /dev/sda)".to_string(),
                required: false,
                default: Some(Value::String("/dev/sda".to_string())),
                example: Some(Value::String("/dev/sda".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "interval_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Measurement interval in milliseconds".to_string(),
                required: false,
                default: Some(Value::Number(1000.into())),
                example: Some(Value::Number(1000.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "disk_iops",
            "parameters": {
                "device": "/dev/sda",
                "interval_ms": 1000
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"Disk IOPS:
Read IOPS: 2345
Write IOPS: 1234
Total IOPS: 3579"#
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
        debug!("Executing disk_iops driver");
        let device = parameters.get("device").and_then(|v| v.as_str()).unwrap_or("/dev/sda");
        let interval = parameters.get("interval_ms").and_then(|v| v.as_u64()).unwrap_or(1000);
        debug!("Getting IOPS for device: {}, interval: {}ms", device, interval);
        let iops = get_disk_iops(device, Duration::from_millis(interval))?;
        let output = format!(
            "Disk IOPS:\n\
             Read IOPS: {}\n\
             Write IOPS: {}\n\
             Total IOPS: {}",
            iops.read_iops,
            iops.write_iops,
            iops.read_iops + iops.write_iops
        );
        info!("Disk IOPS retrieved");
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
fn get_disk_iops(device: &str, interval: Duration) -> DriverResult<DiskIopsInfo> {
    #[cfg(target_os = "linux")]
    {
        let device_name = device.trim_start_matches("/dev/");
        let (read_ops1, write_ops1) = read_diskstats_iops(device_name)?;
        std::thread::sleep(interval);
        let (read_ops2, write_ops2) = read_diskstats_iops(device_name)?;
        let time_diff_sec = interval.as_secs_f64();
        return Ok(DiskIopsInfo {
            read_iops: ((read_ops2 - read_ops1) as f64 / time_diff_sec) as u64,
            write_iops: ((write_ops2 - write_ops1) as f64 / time_diff_sec) as u64,
            total_iops: 0,
        });
    }
    #[cfg(target_os = "windows")]
    {
        return get_windows_disk_iops(device, interval);
    }
    #[cfg(target_os = "macos")]
    {
        return get_macos_disk_iops(device, interval);
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        return Ok(DiskIopsInfo { read_iops: 0, write_iops: 0, total_iops: 0 });
    }
}
#[cfg(target_os = "linux")]
fn read_diskstats_iops(device: &str) -> DriverResult<(u64, u64)> {
    let content = std::fs::read_to_string("/proc/diskstats").map_err(|e| DriverError::execution(format!("Failed to read diskstats: {}", e)))?;
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 14 && parts[2] == device {
            let read_ops = parts[3].parse::<u64>().unwrap_or(0);
            let write_ops = parts[7].parse::<u64>().unwrap_or(0);
            return Ok((read_ops, write_ops));
        }
    }
    return Err(DriverError::execution(format!("Device {} not found in diskstats", device)));
}
#[cfg(target_os = "windows")]
fn get_windows_disk_iops(device: &str, interval: Duration) -> DriverResult<DiskIopsInfo> {
    use std::process::Command;
    let mut read_iops = 0;
    let mut write_iops = 0;
    let output = Command::new("typeperf")
        .args(&["\"\\PhysicalDisk(0 C:)\\Disk Reads/sec\"", "\"\\PhysicalDisk(0 C:)\\Disk Writes/sec\"", "-sc", "1"])
        .output();
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_string();
            let lines: Vec<&str> = output_str.lines().collect();
            if lines.len() >= 2 {
                let data_line = lines[1].trim();
                let parts: Vec<&str> = data_line.split(',').collect();
                if parts.len() >= 3 {
                    if let Ok(ops) = parts[1].trim().parse::<u64>() {
                        read_iops = ops;
                    }
                    if let Ok(ops) = parts[2].trim().parse::<u64>() {
                        write_iops = ops;
                    }
                }
            }
        }
    }
    return Ok(DiskIopsInfo { read_iops, write_iops, total_iops: read_iops + write_iops });
}
#[cfg(target_os = "macos")]
fn get_macos_disk_iops(device: &str, interval: Duration) -> DriverResult<DiskIopsInfo> {
    use std::process::Command;
    let disk_name = device.trim_start_matches("/dev/");
    let mut read_iops = 0;
    let mut write_iops = 0;
    let output = Command::new("iostat").args(&["-d", "-w", &format!("{}", interval.as_secs()), disk_name]).output();
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_string();
            let lines: Vec<&str> = output_str.lines().collect();
            if lines.len() >= 3 {
                let data_line = lines[lines.len() - 1].trim();
                let parts: Vec<&str> = data_line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let xfers = parts.get(2).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                    read_iops = (xfers / 2.0) as u64;
                    write_iops = (xfers / 2.0) as u64;
                }
            }
        }
    }
    return Ok(DiskIopsInfo { read_iops, write_iops, total_iops: read_iops + write_iops });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_disk_iops_metadata() {
        let driver = DiskIopsDriver;
        assert_eq!(driver.name(), "disk_iops");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemDisk);
    }
}
