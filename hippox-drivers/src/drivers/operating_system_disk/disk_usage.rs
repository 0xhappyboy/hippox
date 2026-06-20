//! Disk usage driver

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    drivers::operating_system_disk::common::DiskPartition,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::Disks;

/// Driver for getting disk usage
#[derive(Debug)]
pub struct DiskUsageDriver;

#[async_trait::async_trait]
impl Driver for DiskUsageDriver {
    fn name(&self) -> &str {
        "disk_usage"
    }

    fn description(&self) -> &str {
        "Get disk partition usage including total, used, free space"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to monitor disk space usage and identify full partitions"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "disk_usage",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        r#"Disk Usage:
Device: /dev/sda1 | Mount: / | FS: ext4 | 100 GB / 200 GB (50.0%)
Device: /dev/sda2 | Mount: /home | FS: ext4 | 50 GB / 100 GB (50.0%)"#
            .to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemDisk
    }

    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let partitions = get_disk_usage()?;
        if partitions.is_empty() {
            return Ok("No partitions found".to_string());
        }
        let mut output = String::from("Disk Usage:\n");
        for part in partitions {
            output.push_str(&format!(
                "Device: {} | Mount: {} | FS: {} | {} GB / {} GB ({:.1}%)\n",
                part.device,
                part.mount_point,
                part.file_system,
                part.used_size_gb,
                part.total_size_gb,
                part.usage_percent
            ));
        }
        Ok(output)
    }
}

fn get_disk_usage() -> Result<Vec<DiskPartition>> {
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
            usage_percent: if total > 0 {
                (used as f32 / total as f32) * 100.0
            } else {
                0.0
            },
            is_encrypted: false,
        });
    }
    Ok(partitions)
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
