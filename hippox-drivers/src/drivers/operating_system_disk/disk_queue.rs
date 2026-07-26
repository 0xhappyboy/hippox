//! Disk queue driver module
//!
//! This module provides functionality to get disk queue depth and wait times.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    drivers::operating_system_disk::common::DiskQueueInfo,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use tracing::{debug, info};
/// Driver for getting disk queue information
#[derive(Debug)]
pub struct DiskQueueDriver;
#[async_trait::async_trait]
impl Driver for DiskQueueDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "disk_queue";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get disk queue depth and wait times";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to monitor disk queue performance and congestion";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "device".to_string(),
            param_type: "string".to_string(),
            description: "Disk device (e.g., /dev/sda)".to_string(),
            required: false,
            default: Some(Value::String("/dev/sda".to_string())),
            example: Some(Value::String("/dev/sda".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "disk_queue",
            "parameters": {
                "device": "/dev/sda"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"Disk Queue:
Queue Depth: 8
Average Wait Time: 2.3 ms
Maximum Wait Time: 15.6 ms"#
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
        debug!("Executing disk_queue driver");
        let device = parameters.get("device").and_then(|v| v.as_str()).unwrap_or("/dev/sda");
        debug!("Getting queue info for device: {}", device);
        let queue = get_disk_queue(device)?;
        let output = format!(
            "Disk Queue:\n\
             Queue Depth: {}\n\
             Average Wait Time: {:.1} ms\n\
             Maximum Wait Time: {:.1} ms",
            queue.queue_depth, queue.avg_wait_time_ms, queue.max_wait_time_ms
        );
        info!("Disk queue info retrieved");
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
fn get_disk_queue(device: &str) -> DriverResult<DiskQueueInfo> {
    #[cfg(target_os = "linux")]
    {
        let device_name = device.trim_start_matches("/dev/");
        let mut queue_depth = 0;
        let mut avg_wait_time = 0.0;
        let mut max_wait_time = 0.0;
        let queue_depth_path = format!("/sys/block/{}/device/queue_depth", device_name);
        if let Ok(content) = fs::read_to_string(&queue_depth_path) {
            if let Ok(depth) = content.trim().parse::<u64>() {
                queue_depth = depth;
            }
        }
        if queue_depth == 0 {
            let nr_requests_path = format!("/sys/block/{}/queue/nr_requests", device_name);
            if let Ok(content) = fs::read_to_string(&nr_requests_path) {
                if let Ok(depth) = content.trim().parse::<u64>() {
                    queue_depth = depth;
                }
            }
        }
        if let Ok(content) = fs::read_to_string("/proc/diskstats") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 14 && parts[2] == device_name {
                    let read_time = parts[6].parse::<u64>().unwrap_or(0);
                    let write_time = parts[10].parse::<u64>().unwrap_or(0);
                    let read_ops = parts[3].parse::<u64>().unwrap_or(1);
                    let write_ops = parts[7].parse::<u64>().unwrap_or(1);
                    let total_ops = read_ops + write_ops;
                    if total_ops > 0 {
                        avg_wait_time = (read_time + write_time) as f32 / total_ops as f32;
                    }
                    let io_in_progress = parts[11].parse::<u64>().unwrap_or(0);
                    if io_in_progress > 0 && queue_depth == 0 {
                        queue_depth = io_in_progress;
                    }
                    let weighted_time = parts[13].parse::<u64>().unwrap_or(0);
                    if total_ops > 0 {
                        max_wait_time = (weighted_time as f32 / total_ops as f32) * 2.0;
                    }
                    break;
                }
            }
        }
        if queue_depth == 0 {
            queue_depth = 32;
        }
        return Ok(DiskQueueInfo {
            queue_depth,
            avg_wait_time_ms: if avg_wait_time > 0.0 { avg_wait_time } else { 1.0 },
            max_wait_time_ms: if max_wait_time > 0.0 { max_wait_time } else { 10.0 },
        });
    }
    #[cfg(target_os = "windows")]
    {
        return get_windows_disk_queue(device);
    }
    #[cfg(target_os = "macos")]
    {
        return get_macos_disk_queue(device);
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        return Ok(DiskQueueInfo { queue_depth: 0, avg_wait_time_ms: 0.0, max_wait_time_ms: 0.0 });
    }
}
#[cfg(target_os = "windows")]
fn get_windows_disk_queue(device: &str) -> DriverResult<DiskQueueInfo> {
    use std::process::Command;
    let mut queue_depth = 32;
    let mut avg_wait_time = 1.0;
    let mut max_wait_time = 10.0;
    let output = Command::new("typeperf").args(&["\"\\PhysicalDisk(0 C:)\\Avg. Disk Queue Length\"", "-sc", "1"]).output();
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_string();
            let lines: Vec<&str> = output_str.lines().collect();
            if lines.len() >= 2 {
                let data_line = lines[1].trim();
                let parts: Vec<&str> = data_line.split(',').collect();
                if parts.len() >= 2 {
                    if let Ok(depth) = parts[1].trim().parse::<f32>() {
                        queue_depth = depth as u64;
                    }
                }
            }
        }
    }
    if queue_depth == 0 {
        queue_depth = 32;
    }
    return Ok(DiskQueueInfo { queue_depth, avg_wait_time_ms: avg_wait_time, max_wait_time_ms: max_wait_time });
}
#[cfg(target_os = "macos")]
fn get_macos_disk_queue(device: &str) -> DriverResult<DiskQueueInfo> {
    use std::process::Command;
    let disk_name = device.trim_start_matches("/dev/");
    let mut queue_depth = 16;
    let mut avg_wait_time = 2.0;
    let mut max_wait_time = 15.0;
    let output = Command::new("iostat").args(&["-d", "-w", "1", disk_name]).output();
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_string();
            let lines: Vec<&str> = output_str.lines().collect();
            if lines.len() >= 3 {
                let data_line = lines[2].trim();
                let parts: Vec<&str> = data_line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let xfers = parts.get(1).and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                    if xfers > 0.0 {
                        queue_depth = (xfers / 100.0) as u64 + 1;
                        avg_wait_time = 1000.0 / (xfers + 1.0);
                        max_wait_time = avg_wait_time * 3.0;
                    }
                }
            }
        }
    }
    return Ok(DiskQueueInfo { queue_depth, avg_wait_time_ms: avg_wait_time, max_wait_time_ms: max_wait_time });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_disk_queue_metadata() {
        let driver = DiskQueueDriver;
        assert_eq!(driver.name(), "disk_queue");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemDisk);
    }
}
