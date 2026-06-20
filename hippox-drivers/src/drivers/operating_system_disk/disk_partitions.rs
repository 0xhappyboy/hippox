//! Disk partitions driver

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    drivers::operating_system_disk::common::DiskPartition,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::Disks;

/// Driver for getting disk partitions
#[derive(Debug)]
pub struct DiskPartitionsDriver;

#[async_trait::async_trait]
impl Driver for DiskPartitionsDriver {
    fn name(&self) -> &str {
        "disk_partitions"
    }

    fn description(&self) -> &str {
        "Get all disk partitions with mount points and filesystem types"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to understand disk layout and mount points"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "disk_partitions",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        r#"Disk Partitions:
Device: /dev/sda1 | Mount: /boot | FS: ext4 | Size: 1 GB | Encrypted: No
Device: /dev/sda2 | Mount: / | FS: ext4 | Size: 100 GB | Encrypted: No
Device: /dev/sda3 | Mount: /home | FS: ext4 | Size: 400 GB | Encrypted: Yes"#
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
        let partitions = get_partitions()?;
        if partitions.is_empty() {
            return Ok("No partitions found".to_string());
        }
        let mut output = String::from("Disk Partitions:\n");
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
        Ok(output)
    }
}

fn get_partitions() -> Result<Vec<DiskPartition>> {
    let mut partitions = Vec::new();
    #[cfg(target_os = "linux")]
    {
        // Read from /proc/mounts and /etc/mtab
        if let Ok(content) = std::fs::read_to_string("/proc/mounts") {
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
                    let total_size = match std::fs::metadata(&mount_point) {
                        Ok(_) => {
                            if let Ok(statvfs) = nix::sys::statvfs::statvfs(&mount_point) {
                                (statvfs.blocks() as u64 * statvfs.fragment_size() as u64)
                                    / (1024 * 1024 * 1024)
                            } else {
                                0
                            }
                        }
                        Err(_) => 0,
                    };
                    // Check encryption (simplified)
                    let is_encrypted = device.contains("crypt")
                        || file_system.contains("crypto")
                        || file_system.contains("luks");
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
        // Use sysinfo for non-Linux
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
    Ok(partitions)
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
