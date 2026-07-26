//! Disk I/O driver module
//!
//! This module provides functionality to get disk I/O statistics including
//! read/write speed, throughput, and latency.
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    drivers::operating_system_disk::common::DiskIoStats,
    types::{Driver, DriverParameter},
};
/// Driver for getting disk I/O statistics
#[derive(Debug)]
pub struct DiskIoDriver;
#[async_trait::async_trait]
impl Driver for DiskIoDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "disk_io";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get disk I/O statistics including read/write speed and throughput";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to monitor disk performance and identify I/O bottlenecks";
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
                name: "interval_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Measurement interval in milliseconds".to_string(),
                required: false,
                default: Some(Value::Number(1000.into())),
                example: Some(Value::Number(2000.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "disk_io",
            "parameters": {
                "device": "/dev/sda",
                "interval_ms": 1000
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"Disk I/O Statistics:
Read: 50.2 MB/s | 2345 ops/s
Write: 30.1 MB/s | 1234 ops/s
Read Latency: 2.3 ms
Write Latency: 1.8 ms"#
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
        debug!("Executing disk_io driver");
        let device = parameters.get("device").and_then(|v| v.as_str()).unwrap_or("/dev/sda");
        let interval = parameters.get("interval_ms").and_then(|v| v.as_u64()).unwrap_or(1000);
        debug!("Getting I/O stats for device: {}, interval: {}ms", device, interval);
        let stats = get_disk_io(device, Duration::from_millis(interval))?;
        let output = format!(
            "Disk I/O Statistics:\n\
             Read: {:.1} MB/s | {} ops/s\n\
             Write: {:.1} MB/s | {} ops/s\n\
             Read Latency: {:.1} ms\n\
             Write Latency: {:.1} ms",
            stats.read_bytes_per_sec as f64 / (1024.0 * 1024.0),
            stats.read_ops_per_sec,
            stats.write_bytes_per_sec as f64 / (1024.0 * 1024.0),
            stats.write_ops_per_sec,
            stats.avg_read_latency_ms,
            stats.avg_write_latency_ms
        );
        info!("Disk I/O statistics retrieved");
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
fn get_disk_io(device: &str, interval: Duration) -> DriverResult<DiskIoStats> {
    #[cfg(target_os = "linux")]
    {
        let device_name = device.trim_start_matches("/dev/");
        let (read_ops1, write_ops1, read_bytes1, write_bytes1, read_time1, write_time1) = read_diskstats(device_name)?;
        std::thread::sleep(interval);
        let (read_ops2, write_ops2, read_bytes2, write_bytes2, read_time2, write_time2) = read_diskstats(device_name)?;
        let time_diff_sec = interval.as_secs_f64();
        return Ok(DiskIoStats {
            read_bytes_per_sec: ((read_bytes2 - read_bytes1) as f64 / time_diff_sec) as u64,
            write_bytes_per_sec: ((write_bytes2 - write_bytes1) as f64 / time_diff_sec) as u64,
            read_ops_per_sec: ((read_ops2 - read_ops1) as f64 / time_diff_sec) as u64,
            write_ops_per_sec: ((write_ops2 - write_ops1) as f64 / time_diff_sec) as u64,
            avg_read_latency_ms: if read_ops2 > read_ops1 { (read_time2 - read_time1) as f32 / (read_ops2 - read_ops1) as f32 } else { 0.0 },
            avg_write_latency_ms: if write_ops2 > write_ops1 { (write_time2 - write_time1) as f32 / (write_ops2 - write_ops1) as f32 } else { 0.0 },
        });
    }
    #[cfg(target_os = "windows")]
    {
        return get_windows_disk_io(device, interval);
    }
    #[cfg(target_os = "macos")]
    {
        return get_macos_disk_io(device, interval);
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        return Ok(DiskIoStats {
            read_bytes_per_sec: 0,
            write_bytes_per_sec: 0,
            read_ops_per_sec: 0,
            write_ops_per_sec: 0,
            avg_read_latency_ms: 0.0,
            avg_write_latency_ms: 0.0,
        });
    }
}
#[cfg(target_os = "linux")]
fn read_diskstats(device: &str) -> DriverResult<(u64, u64, u64, u64, u64, u64)> {
    let content = std::fs::read_to_string("/proc/diskstats").map_err(|e| DriverError::execution(format!("Failed to read diskstats: {}", e)))?;
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 14 && parts[2] == device {
            let read_ops = parts[3].parse::<u64>().unwrap_or(0);
            let read_sectors = parts[5].parse::<u64>().unwrap_or(0);
            let read_time = parts[6].parse::<u64>().unwrap_or(0);
            let write_ops = parts[7].parse::<u64>().unwrap_or(0);
            let write_sectors = parts[9].parse::<u64>().unwrap_or(0);
            let write_time = parts[10].parse::<u64>().unwrap_or(0);
            let read_bytes = read_sectors * 512;
            let write_bytes = write_sectors * 512;
            return Ok((read_ops, write_ops, read_bytes, write_bytes, read_time, write_time));
        }
    }
    return Err(DriverError::execution(format!("Device {} not found in diskstats", device)));
}
#[cfg(target_os = "windows")]
fn get_windows_disk_io(device: &str, interval: Duration) -> DriverResult<DiskIoStats> {
    use std::process::Command;
    let disk_label = if device.contains("C:") || device.contains("PhysicalDrive0") { "C:".to_string() } else { "C:".to_string() };
    let mut stats = DiskIoStats {
        read_bytes_per_sec: 0,
        write_bytes_per_sec: 0,
        read_ops_per_sec: 0,
        write_ops_per_sec: 0,
        avg_read_latency_ms: 0.0,
        avg_write_latency_ms: 0.0,
    };
    let output = Command::new("typeperf")
        .args(&[
            "\"\\PhysicalDisk(0 C:)\\Disk Read Bytes/sec\"",
            "\"\\PhysicalDisk(0 C:)\\Disk Write Bytes/sec\"",
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
                if parts.len() >= 5 {
                    if let Ok(bytes) = parts[1].trim().parse::<u64>() {
                        stats.read_bytes_per_sec = bytes;
                    }
                    if let Ok(bytes) = parts[2].trim().parse::<u64>() {
                        stats.write_bytes_per_sec = bytes;
                    }
                    if let Ok(ops) = parts[3].trim().parse::<u64>() {
                        stats.read_ops_per_sec = ops;
                    }
                    if let Ok(ops) = parts[4].trim().parse::<u64>() {
                        stats.write_ops_per_sec = ops;
                    }
                }
            }
        }
    }
    return Ok(stats);
}
#[cfg(target_os = "macos")]
fn get_macos_disk_io(device: &str, interval: Duration) -> DriverResult<DiskIoStats> {
    use std::process::Command;
    let disk_name = device.trim_start_matches("/dev/");
    let mut stats = DiskIoStats {
        read_bytes_per_sec: 0,
        write_bytes_per_sec: 0,
        read_ops_per_sec: 0,
        write_ops_per_sec: 0,
        avg_read_latency_ms: 0.0,
        avg_write_latency_ms: 0.0,
    };
    let output = Command::new("iostat").args(&["-d", "-w", &format!("{}", interval.as_secs()), disk_name]).output();
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_string();
            let lines: Vec<&str> = output_str.lines().collect();
            if lines.len() >= 3 {
                let data_line = lines[lines.len() - 1].trim();
                let parts: Vec<&str> = data_line.split_whitespace().collect();
                if parts.len() >= 6 {
                    let xfers = parts.get(2).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                    let mb_s = parts.get(3).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                    let total_bytes = (mb_s * 1024.0 * 1024.0) as u64;
                    stats.read_bytes_per_sec = total_bytes / 2;
                    stats.write_bytes_per_sec = total_bytes / 2;
                    stats.read_ops_per_sec = (xfers / 2.0) as u64;
                    stats.write_ops_per_sec = (xfers / 2.0) as u64;
                }
            }
        }
    }
    return Ok(stats);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_disk_io_metadata() {
        let driver = DiskIoDriver;
        assert_eq!(driver.name(), "disk_io");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemDisk);
    }
}
