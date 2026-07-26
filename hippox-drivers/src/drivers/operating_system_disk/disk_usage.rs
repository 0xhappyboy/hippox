//! Disk usage driver module
//!
//! This module provides functionality to get disk partition usage including
//! total, used, and free space.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    drivers::operating_system_disk::common::DiskPartition,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::Disks;
use tracing::{debug, info};
/// Driver for getting disk usage
#[derive(Debug)]
pub struct DiskUsageDriver;
#[async_trait::async_trait]
impl Driver for DiskUsageDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "disk_usage";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get disk partition usage including total, used, free space";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to monitor disk space usage and identify full partitions";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "disk_usage",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"Disk Usage:
Device: /dev/sda1 | Mount: / | FS: ext4 | 100 GB / 200 GB (50.0%)
Device: /dev/sda2 | Mount: /home | FS: ext4 | 50 GB / 100 GB (50.0%)"#
            .to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemDisk;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing disk_usage driver");
        let partitions = get_disk_usage()?;
        if partitions.is_empty() {
            info!("No partitions found");
            return Ok("No partitions found".to_string());
        }
        let mut output = String::from("Disk Usage:\n");
        let partitions_size = partitions.len();
        for part in partitions {
            output.push_str(&format!(
                "Device: {} | Mount: {} | FS: {} | {} GB / {} GB ({:.1}%)\n",
                part.device, part.mount_point, part.file_system, part.used_size_gb, part.total_size_gb, part.usage_percent
            ));
        }
        info!("Disk usage retrieved for {} partitions", partitions_size);
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
fn get_disk_usage() -> DriverResult<Vec<DiskPartition>> {
    let mut partitions = Vec::new();
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        let total = disk.total_space();
        let used = disk.total_space() - disk.available_space();
        let available = disk.available_space();
        partitions.push(DiskPartition {
            device: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            file_system: format!("{:?}", disk.file_system()),
            total_size_gb: total / (1024 * 1024 * 1024),
            used_size_gb: used / (1024 * 1024 * 1024),
            free_size_gb: available / (1024 * 1024 * 1024),
            usage_percent: if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 },
            is_encrypted: false,
        });
    }
    return Ok(partitions);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_disk_usage_metadata() {
        let driver = DiskUsageDriver;
        assert_eq!(driver.name(), "disk_usage");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemDisk);
    }
}
