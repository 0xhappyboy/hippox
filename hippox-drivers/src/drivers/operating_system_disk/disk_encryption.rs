//! Disk encryption driver module
//!
//! This module provides functionality to check if disk partitions are
//! encrypted with BitLocker (Windows), LUKS (Linux), or FileVault (macOS).
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    drivers::operating_system_disk::common::DiskPartition,
    types::{Driver, DriverParameter},
};
/// Driver for checking disk encryption status
#[derive(Debug)]
pub struct DiskEncryptionDriver;
#[async_trait::async_trait]
impl Driver for DiskEncryptionDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "disk_encryption";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Check if disk partitions are encrypted (BitLocker, LUKS, FileVault)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to verify disk encryption for security compliance";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "disk_encryption",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"Disk Encryption Status:
Partition: /dev/sda1 | Mount: /boot | Encrypted: No
Partition: /dev/sda2 | Mount: / | Encrypted: Yes (LUKS)
Partition: /dev/sda3 | Mount: /home | Encrypted: Yes (LUKS)"#
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
        debug!("Executing disk_encryption driver");
        let partitions = check_encryption()?;
        if partitions.is_empty() {
            info!("No partitions found");
            return Ok("No partitions found".to_string());
        }
        let mut output = String::from("Disk Encryption Status:\n");
        for part in partitions {
            output.push_str(&format!(
                "Partition: {} | Mount: {} | Encrypted: {}\n",
                part.device,
                part.mount_point,
                if part.is_encrypted { format!("Yes ({})", part.file_system) } else { "No".to_string() }
            ));
        }
        info!("Disk encryption status checked");
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
fn check_encryption() -> DriverResult<Vec<DiskPartition>> {
    let mut partitions = Vec::new();
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/mounts") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let device = parts[0];
                    let mount_point = parts[1];
                    let fs = parts[2];
                    let is_encrypted = device.contains("crypt") || device.contains("luks") || fs.contains("crypto") || fs.contains("luks");
                    let is_luks =
                        if let Ok(output) = Command::new("cryptsetup").args(&["isLuks", device]).output() { output.status.success() } else { false };
                    partitions.push(DiskPartition {
                        device: device.to_string(),
                        mount_point: mount_point.to_string(),
                        file_system: fs.to_string(),
                        total_size_gb: 0,
                        used_size_gb: 0,
                        free_size_gb: 0,
                        usage_percent: 0.0,
                        is_encrypted: is_encrypted || is_luks,
                    });
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        partitions = get_windows_encryption_status()?;
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("fdesetup").args(&["status"]).output() {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    let is_encrypted = output_str.contains("FileVault is On");
                    if let Ok(content) = std::fs::read_to_string("/etc/fstab") {
                        for line in content.lines() {
                            if line.contains("/dev/") {
                                let parts: Vec<&str> = line.split_whitespace().collect();
                                if parts.len() >= 2 {
                                    partitions.push(DiskPartition {
                                        device: parts[0].to_string(),
                                        mount_point: parts[1].to_string(),
                                        file_system: "APFS".to_string(),
                                        total_size_gb: 0,
                                        used_size_gb: 0,
                                        free_size_gb: 0,
                                        usage_percent: 0.0,
                                        is_encrypted,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return Ok(partitions);
}
#[cfg(target_os = "windows")]
fn get_windows_encryption_status() -> DriverResult<Vec<DiskPartition>> {
    use std::process::Command;
    let mut partitions = Vec::new();
    let output = Command::new("manage-bde").args(&["-status"]).output();
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout).to_string();
            let mut current_volume = String::new();
            let mut current_encrypted = false;
            let mut current_fs = "NTFS".to_string();
            for line in output_str.lines() {
                let trimmed = line.trim();
                if trimmed.contains("Volume") && trimmed.contains(":") {
                    if !current_volume.is_empty() {
                        partitions.push(DiskPartition {
                            device: current_volume.clone(),
                            mount_point: current_volume.clone(),
                            file_system: current_fs.clone(),
                            total_size_gb: 0,
                            used_size_gb: 0,
                            free_size_gb: 0,
                            usage_percent: 0.0,
                            is_encrypted: current_encrypted,
                        });
                    }
                    if let Some(vol) = trimmed.split(':').nth(1) {
                        current_volume = vol.trim().to_string();
                    }
                    current_encrypted = false;
                }
                if trimmed.contains("Protection Status") {
                    if trimmed.contains("Protection On") {
                        current_encrypted = true;
                    }
                }
                if trimmed.contains("Conversion Status") {
                    if trimmed.contains("Fully Decrypted") || trimmed.contains("Not Encrypted") {
                        current_encrypted = false;
                    }
                }
            }
            if !current_volume.is_empty() {
                partitions.push(DiskPartition {
                    device: current_volume.clone(),
                    mount_point: current_volume,
                    file_system: current_fs,
                    total_size_gb: 0,
                    used_size_gb: 0,
                    free_size_gb: 0,
                    usage_percent: 0.0,
                    is_encrypted: current_encrypted,
                });
            }
        }
    }
    if partitions.is_empty() {
        let output =
            Command::new("powershell").args(&["-Command", "Get-BitLockerVolume | Select-Object MountPoint, ProtectionStatus, VolumeType"]).output();
        if let Ok(output) = output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout).to_string();
                let lines: Vec<&str> = output_str.lines().collect();
                if lines.len() > 1 {
                    for line in &lines[1..] {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        let parts: Vec<&str> = trimmed.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let mount = parts[0].to_string();
                            let is_encrypted = parts.get(1).map(|s| *s == "On").unwrap_or(false);
                            partitions.push(DiskPartition {
                                device: mount.clone(),
                                mount_point: mount,
                                file_system: "NTFS".to_string(),
                                total_size_gb: 0,
                                used_size_gb: 0,
                                free_size_gb: 0,
                                usage_percent: 0.0,
                                is_encrypted,
                            });
                        }
                    }
                }
            }
        }
    }
    return Ok(partitions);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_disk_encryption_metadata() {
        let driver = DiskEncryptionDriver;
        assert_eq!(driver.name(), "disk_encryption");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemDisk);
    }
}
