//! Disk information driver module
//!
//! This module provides functionality to get detailed disk information
//! including model, serial number, interface type, and size.
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    drivers::operating_system_disk::common::DiskInfo,
    types::{Driver, DriverParameter},
};
/// Driver for getting disk information
#[derive(Debug)]
pub struct DiskInfoDriver;
#[async_trait::async_trait]
impl Driver for DiskInfoDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "disk_info";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get detailed disk information including model, serial, interface, and size";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to get disk specifications and hardware details";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "disk_info",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"Disk Information:
Name: /dev/sda
Model: Samsung SSD 860 EVO 500GB
Serial: S3Z9NB0M123456
Interface: SATA
Transfer Mode: SATA III (6 Gbps)
Size: 500 GB
Type: SSD
Removable: No"#
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
        debug!("Executing disk_info driver");
        let disks = get_disk_info()?;
        if disks.is_empty() {
            info!("No disks detected");
            return Ok("No disks detected".to_string());
        }
        let mut output = String::from("Disk Information:\n");
        for (i, disk) in disks.iter().enumerate() {
            if i > 0 {
                output.push_str("\n");
            }
            output.push_str(&format!("Name: {}\n", disk.name));
            output.push_str(&format!("Model: {}\n", disk.model));
            output.push_str(&format!("Serial: {}\n", disk.serial_number));
            output.push_str(&format!("Interface: {}\n", disk.interface_type));
            output.push_str(&format!("Transfer Mode: {}\n", disk.transfer_mode));
            output.push_str(&format!("Size: {} GB\n", disk.size_gb));
            output.push_str(&format!("Type: {}\n", if disk.is_ssd { "SSD" } else { "HDD" }));
            output.push_str(&format!("Removable: {}\n", if disk.is_removable { "Yes" } else { "No" }));
        }
        info!("Disk information retrieved for {} disks", disks.len());
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
fn get_disk_info() -> DriverResult<Vec<DiskInfo>> {
    let mut disks = Vec::new();
    #[cfg(target_os = "linux")]
    {
        let block_path = "/sys/block";
        if let Ok(entries) = std::fs::read_dir(block_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("loop") || name.starts_with("ram") {
                        continue;
                    }
                    let device_path = format!("/dev/{}", name);
                    let model_path = format!("/sys/block/{}/device/model", name);
                    let model = std::fs::read_to_string(&model_path).map(|s| s.trim().to_string()).unwrap_or_else(|_| "Unknown".to_string());
                    let serial_path = format!("/sys/block/{}/device/serial", name);
                    let serial = std::fs::read_to_string(&serial_path).map(|s| s.trim().to_string()).unwrap_or_else(|_| "Unknown".to_string());
                    let size_path = format!("/sys/block/{}/size", name);
                    let size_sectors = std::fs::read_to_string(&size_path).map(|s| s.trim().parse::<u64>().unwrap_or(0)).unwrap_or(0);
                    let size_gb = (size_sectors * 512) / (1024 * 1024 * 1024);
                    let rotational_path = format!("/sys/block/{}/queue/rotational", name);
                    let is_ssd = std::fs::read_to_string(&rotational_path).map(|s| s.trim() == "0").unwrap_or(false);
                    let removable_path = format!("/sys/block/{}/removable", name);
                    let is_removable = std::fs::read_to_string(&removable_path).map(|s| s.trim() == "1").unwrap_or(false);
                    let interface = if name.starts_with("sd") {
                        "SATA".to_string()
                    } else if name.starts_with("nvme") {
                        "NVMe".to_string()
                    } else if name.starts_with("vd") {
                        "VirtIO".to_string()
                    } else {
                        "Unknown".to_string()
                    };
                    let transfer_mode = if interface == "SATA" {
                        if let Ok(link_speed) = std::fs::read_to_string(format!("/sys/block/{}/device/sata_link_speed", name)) {
                            format!("SATA {}", link_speed.trim())
                        } else {
                            "SATA III (6 Gbps)".to_string()
                        }
                    } else if interface == "NVMe" {
                        "NVMe (PCIe)".to_string()
                    } else {
                        "Unknown".to_string()
                    };
                    disks.push(DiskInfo {
                        name: device_path,
                        model: model.trim().to_string(),
                        serial_number: serial.trim().to_string(),
                        interface_type: interface,
                        transfer_mode,
                        size_gb,
                        is_ssd,
                        is_removable,
                    });
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        disks = get_windows_disks()?;
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("system_profiler").args(&["SPStorageDataType", "-json"]).output() {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
                        if let Some(storage) = json.get("SPStorageDataType").and_then(|v| v.as_array()) {
                            for device in storage {
                                let name = device.get("_name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                                disks.push(DiskInfo {
                                    name,
                                    model: "Unknown".to_string(),
                                    serial_number: "Unknown".to_string(),
                                    interface_type: "Unknown".to_string(),
                                    transfer_mode: "Unknown".to_string(),
                                    size_gb: 0,
                                    is_ssd: false,
                                    is_removable: false,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        disks.push(DiskInfo {
            name: format!("/dev/disk0 (OS: {})", std::env::consts::OS),
            model: "Unknown".to_string(),
            serial_number: "Unknown".to_string(),
            interface_type: "Unknown".to_string(),
            transfer_mode: "Unknown".to_string(),
            size_gb: 0,
            is_ssd: false,
            is_removable: false,
        });
    }
    return Ok(disks);
}
#[cfg(target_os = "windows")]
fn get_windows_disks() -> DriverResult<Vec<DiskInfo>> {
    use std::process::Command;
    let mut disks = Vec::new();
    let output = Command::new("wmic").args(&["diskdrive", "get", "Name,Model,SerialNumber,InterfaceType,MediaType,Size"]).output();
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
                        let name = parts.first().unwrap_or(&"").to_string();
                        let model = parts.get(1).unwrap_or(&"").to_string();
                        let serial = parts.get(2).unwrap_or(&"").to_string();
                        let interface = parts.get(3).unwrap_or(&"").to_string();
                        let size_str = parts.last().unwrap_or(&"").to_string();
                        let size_gb = size_str.parse::<u64>().unwrap_or(0) / (1024 * 1024 * 1024);
                        let is_ssd = model.to_lowercase().contains("ssd")
                            || model.to_lowercase().contains("nvme")
                            || interface.to_lowercase().contains("nvme");
                        let interface_lower = interface.to_lowercase();
                        disks.push(DiskInfo {
                            name: if name.is_empty() { "Unknown".to_string() } else { name },
                            model: if model.is_empty() { "Unknown".to_string() } else { model },
                            serial_number: if serial.is_empty() { "Unknown".to_string() } else { serial },
                            interface_type: if interface.is_empty() { "Unknown".to_string() } else { interface },
                            transfer_mode: if interface_lower.to_lowercase().contains("sata") {
                                "SATA III (6 Gbps)".to_string()
                            } else if interface_lower.to_lowercase().contains("nvme") {
                                "NVMe (PCIe)".to_string()
                            } else {
                                "Unknown".to_string()
                            },
                            size_gb,
                            is_ssd,
                            is_removable: false,
                        });
                    }
                }
            }
        }
    }
    if disks.is_empty() {
        let output = Command::new("powershell")
            .args(&["-Command", "Get-PhysicalDisk | Select-Object FriendlyName, SerialNumber, MediaType, Size, BusType"])
            .output();
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
                            let name = parts.first().unwrap_or(&"").to_string();
                            let size_str = parts.get(3).unwrap_or(&"0").to_string();
                            let size_gb = size_str.parse::<u64>().unwrap_or(0) / (1024 * 1024 * 1024);
                            let media_type = parts.get(2).unwrap_or(&"").to_string();
                            let is_ssd = media_type.to_lowercase().contains("ssd") || media_type.to_lowercase().contains("nvme");
                            disks.push(DiskInfo {
                                name: format!("PhysicalDisk ({})", name),
                                model: if name.is_empty() { "Unknown".to_string() } else { name },
                                serial_number: parts.get(1).unwrap_or(&"Unknown").to_string(),
                                interface_type: parts.get(4).unwrap_or(&"Unknown").to_string(),
                                transfer_mode: "Unknown".to_string(),
                                size_gb,
                                is_ssd,
                                is_removable: false,
                            });
                        }
                    }
                }
            }
        }
    }
    if disks.is_empty() {
        disks.push(DiskInfo {
            name: "C:".to_string(),
            model: "Unknown".to_string(),
            serial_number: "Unknown".to_string(),
            interface_type: "Unknown".to_string(),
            transfer_mode: "Unknown".to_string(),
            size_gb: 0,
            is_ssd: false,
            is_removable: false,
        });
    }
    return Ok(disks);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_disk_info_metadata() {
        let driver = DiskInfoDriver;
        assert_eq!(driver.name(), "disk_info");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemDisk);
    }
}
