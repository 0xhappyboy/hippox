//! Disk IOPS driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    drivers::operating_system_disk::common::DiskIopsInfo,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;
/// Driver for getting disk IOPS (Input/Output Operations Per Second)
#[derive(Debug)]
pub struct DiskIopsDriver;
#[async_trait::async_trait]
impl Driver for DiskIopsDriver {
    fn name(&self) -> &str {
        "disk_iops"
    }
    fn description(&self) -> &str {
        "Get disk IOPS (Input/Output Operations Per Second) for read and write"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to measure disk performance in IOPS"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "disk_iops",
            "parameters": {
                "device": "/dev/sda",
                "interval_ms": 1000
            }
        })
    }
    fn example_output(&self) -> String {
        r#"Disk IOPS:
Read IOPS: 2345
Write IOPS: 1234
Total IOPS: 3579"#
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
        let interval = parameters
            .get("interval_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000);
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
        Ok(output)
    }
}
fn get_disk_iops(device: &str, interval: Duration) -> Result<DiskIopsInfo> {
    #[cfg(target_os = "linux")]
    {
        let device_name = device.trim_start_matches("/dev/");
        let (read_ops1, write_ops1) = read_diskstats_iops(device_name)?;
        std::thread::sleep(interval);
        let (read_ops2, write_ops2) = read_diskstats_iops(device_name)?;
        let time_diff_sec = interval.as_secs_f64();
        Ok(DiskIopsInfo {
            read_iops: ((read_ops2 - read_ops1) as f64 / time_diff_sec) as u64,
            write_iops: ((write_ops2 - write_ops1) as f64 / time_diff_sec) as u64,
            total_iops: 0,
        })
    }
    #[cfg(target_os = "windows")]
    {
        get_windows_disk_iops(device, interval)
    }
    #[cfg(target_os = "macos")]
    {
        get_macos_disk_iops(device, interval)
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Ok(DiskIopsInfo {
            read_iops: 0,
            write_iops: 0,
            total_iops: 0,
        })
    }
}
#[cfg(target_os = "linux")]
fn read_diskstats_iops(device: &str) -> Result<(u64, u64)> {
    let content = std::fs::read_to_string("/proc/diskstats")?;
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 14 && parts[2] == device {
            let read_ops = parts[3].parse::<u64>().unwrap_or(0);
            let write_ops = parts[7].parse::<u64>().unwrap_or(0);
            return Ok((read_ops, write_ops));
        }
    }
    Err(anyhow::anyhow!("Device not found in diskstats"))
}
#[cfg(target_os = "windows")]
fn get_windows_disk_iops(device: &str, interval: Duration) -> Result<DiskIopsInfo> {
    use std::process::Command;
    let disk_index = if device.contains("PhysicalDrive") {
        device
            .trim_start_matches("\\\\.\\PhysicalDrive")
            .parse::<u32>()
            .unwrap_or(0)
    } else if device.contains("C:") {
        0
    } else {
        0
    };
    let disk_label = if disk_index == 0 {
        "C:".to_string()
    } else {
        format!("{}", disk_index)
    };
    let output = Command::new("powershell")
    .args(&[
        "-Command",
        &format!(
            "Get-CimInstance -Namespace root/cimv2 -ClassName Win32_PerfFormattedData_PerfDisk_PhysicalDisk | Where-Object {{ $_.Name -match '{}' }} | Select-Object DiskReadsPerSec, DiskWritesPerSec",
            disk_label
        )
    ])
    .output();
    let mut read_iops = 0;
    let mut write_iops = 0;
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_string();
            for line in output_str.lines() {
                if line.contains("DiskReadsPerSec") {
                    if let Some(val) = line.split(':').nth(1) {
                        if let Ok(ops) = val.trim().parse::<u64>() {
                            read_iops = ops;
                        }
                    }
                }
                if line.contains("DiskWritesPerSec") {
                    if let Some(val) = line.split(':').nth(1) {
                        if let Ok(ops) = val.trim().parse::<u64>() {
                            write_iops = ops;
                        }
                    }
                }
            }
        }
    }
    if read_iops == 0 && write_iops == 0 {
        let output = Command::new("typeperf")
            .args(&[
                "\"\\PhysicalDisk(0 C:)\\Disk Reads/sec\"",
                "\"\\PhysicalDisk(0 C:)\\Disk Writes/sec\"",
                "-sc",
                "1",
            ])
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
    }
    Ok(DiskIopsInfo {
        read_iops,
        write_iops,
        total_iops: read_iops + write_iops,
    })
}
#[cfg(target_os = "macos")]
fn get_macos_disk_iops(device: &str, interval: Duration) -> Result<DiskIopsInfo> {
    use std::process::Command;
    let disk_name = device.trim_start_matches("/dev/");
    let mut read_iops = 0;
    let mut write_iops = 0;
    let output = Command::new("iostat")
        .args(&["-d", "-w", &format!("{}", interval.as_secs()), disk_name])
        .output();
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_string();
            let lines: Vec<&str> = output_str.lines().collect();
            if lines.len() >= 3 {
                let data_line = lines[lines.len() - 1].trim();
                let parts: Vec<&str> = data_line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let xfers = parts
                        .get(2)
                        .and_then(|s| s.parse::<f32>().ok())
                        .unwrap_or(0.0);
                    read_iops = (xfers / 2.0) as u64;
                    write_iops = (xfers / 2.0) as u64;
                }
            }
        }
    }
    let output = Command::new("sysctl")
        .args(&["-n", &format!("kern.diskstats.{}", disk_name)])
        .output();
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_string();
            let parts: Vec<&str> = output_str.split_whitespace().collect();
            if parts.len() >= 9 {
                if let Ok(ops) = parts.get(1).and_then(|s| s.parse::<u64>().ok()) {
                    read_iops = ops / interval.as_secs();
                }
                if let Ok(ops) = parts.get(4).and_then(|s| s.parse::<u64>().ok()) {
                    write_iops = ops / interval.as_secs();
                }
            }
        }
    }
    Ok(DiskIopsInfo {
        read_iops,
        write_iops,
        total_iops: read_iops + write_iops,
    })
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
