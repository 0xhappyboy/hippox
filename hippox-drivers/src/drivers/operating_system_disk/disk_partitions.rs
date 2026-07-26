//! Disk partitions driver module
//!
//! This module provides functionality to get all disk partitions with
//! mount points and filesystem types.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    drivers::operating_system_disk::common::DiskPartition,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use sysinfo::Disks;
use tracing::{debug, info};
/// Driver for getting disk partitions
#[derive(Debug)]
pub struct DiskPartitionsDriver;
#[async_trait::async_trait]
impl Driver for DiskPartitionsDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "disk_partitions";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get all disk partitions with mount points and filesystem types";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to understand disk layout and mount points";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "disk_partitions",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"Disk Partitions:
Device: /dev/sda1 | Mount: /boot | FS: ext4 | Size: 1 GB | Encrypted: No
Device: /dev/sda2 | Mount: / | FS: ext4 | Size: 100 GB | Encrypted: No
Device: /dev/sda3 | Mount: /home | FS: ext4 | Size: 400 GB | Encrypted: Yes"#
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
        debug!("Executing disk_partitions driver");
        let partitions = get_partitions()?;
        if partitions.is_empty() {
            info!("No partitions found");
            return Ok("No partitions found".to_string());
        }
        let mut output = String::from("Disk Partitions:\n");
        let partitions_size = partitions.len();
        for part in partitions {
            output.push_str(&format!(
                "Device: {} | Mount: {} | FS: {} | Size: {} GB | Encrypted: {}\n",
                part.device,
                part.mount_point,
                part.file_system,
                part.total_size_gb,
                if part.is_encrypted { "Yes" } else { "No" }
            ));
        }
        info!("Retrieved {} partitions", partitions_size);
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
fn get_partitions() -> DriverResult<Vec<DiskPartition>> {
    let mut partitions = Vec::new();
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = fs::read_to_string("/proc/mounts") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let device = parts[0].to_string();
                    let mount_point = parts[1].to_string();
                    let file_system = parts[2].to_string();
                    // Skip non-disk mounts
                    if device.starts_with("tmpfs")
                        || device.starts_with("devtmpfs")
                        || device.starts_with("cgroup")
                        || device.starts_with("sysfs")
                        || device.starts_with("proc")
                        || device.starts_with("devpts")
                        || device.starts_with("securityfs")
                        || device.starts_with("pstore")
                        || device.starts_with("mqueue")
                        || device.starts_with("hugetlbfs")
                        || device.starts_with("sunrpc")
                        || device.starts_with("binfmt_misc")
                        || device.starts_with("debugfs")
                        || device.starts_with("tracefs")
                        || device.starts_with("fusectl")
                        || device.starts_with("configfs")
                    {
                        continue;
                    }
                    // Get size info
                    let total_size = match fs::metadata(&mount_point) {
                        Ok(_) => {
                            if let Ok(statvfs) = nix::sys::statvfs::statvfs(&mount_point) {
                                (statvfs.blocks() as u64 * statvfs.fragment_size() as u64) / (1024 * 1024 * 1024)
                            } else {
                                0
                            }
                        }
                        Err(_) => 0,
                    };
                    let is_encrypted = device.contains("crypt") || file_system.contains("crypto") || file_system.contains("luks");
                    partitions.push(DiskPartition {
                        device,
                        mount_point,
                        file_system,
                        total_size_gb: total_size,
                        used_size_gb: 0,
                        free_size_gb: 0,
                        usage_percent: 0.0,
                        is_encrypted,
                    });
                }
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            partitions.push(DiskPartition {
                device: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                file_system: format!("{:?}", disk.file_system()),
                total_size_gb: disk.total_space() / (1024 * 1024 * 1024),
                used_size_gb: (disk.total_space() - disk.available_space()) / (1024 * 1024 * 1024),
                free_size_gb: disk.available_space() / (1024 * 1024 * 1024),
                usage_percent: 0.0,
                is_encrypted: false,
            });
        }
    }
    return Ok(partitions);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_disk_partitions_metadata() {
        let driver = DiskPartitionsDriver;
        assert_eq!(driver.name(), "disk_partitions");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemDisk);
    }
}
