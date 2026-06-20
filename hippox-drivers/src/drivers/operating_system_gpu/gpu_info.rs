//! GPU information driver

use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for getting GPU information
#[derive(Debug)]
pub struct GpuInfoDriver;

#[async_trait::async_trait]
impl Driver for GpuInfoDriver {
    fn name(&self) -> &str {
        "gpu_info"
    }

    fn description(&self) -> &str {
        "Get detailed GPU information including model, vendor, driver, and memory"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get GPU specifications and capabilities"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "gpu_info",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        r#"GPU Information:
Name: NVIDIA GeForce RTX 3080
Vendor: NVIDIA Corporation
Driver Version: 525.125.06
Total Memory: 10240 MB
Memory Type: GDDR6X
PCIe Speed: 16 GT/s
PCIe Width: x16"#
            .to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemGpu
    }

    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let gpus = detect_gpus()?;

        if gpus.is_empty() {
            return Ok("No GPU detected".to_string());
        }

        let mut output = String::from("GPU Information:\n");
        for (i, gpu) in gpus.iter().enumerate() {
            if i > 0 {
                output.push_str("\n");
            }
            output.push_str(&format!("Name: {}\n", gpu.name));
            output.push_str(&format!("Vendor: {}\n", gpu.vendor));
            output.push_str(&format!("Driver Version: {}\n", gpu.driver_version));
            output.push_str(&format!("Total Memory: {} MB\n", gpu.total_memory_mb));
            output.push_str(&format!("Memory Type: {}\n", gpu.memory_type));
            output.push_str(&format!("PCIe Speed: {}\n", gpu.pcie_speed));
            output.push_str(&format!("PCIe Width: x{}\n", gpu.pcie_width));
            if let Some(bios) = &gpu.bios_version {
                output.push_str(&format!("BIOS Version: {}\n", bios));
            }
            if let Some(serial) = &gpu.serial_number {
                output.push_str(&format!("Serial: {}\n", serial));
            }
        }

        Ok(output)
    }
}

#[derive(Debug, Clone)]
struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub driver_version: String,
    pub total_memory_mb: u64,
    pub memory_type: String,
    pub pcie_speed: String,
    pub pcie_width: u8,
    pub bios_version: Option<String>,
    pub serial_number: Option<String>,
}

fn detect_gpus() -> Result<Vec<GpuInfo>> {
    #[cfg(target_os = "linux")]
    {
        let mut gpus = Vec::new();
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(&["--query-gpu", "name,driver_version,memory.total,memory.type,pcie.link.gen.current,pcie.link.width.current,bios_version,serial"])
            .args(&["--format", "csv,noheader"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() >= 6 {
                            gpus.push(GpuInfo {
                                name: parts[0].trim().to_string(),
                                vendor: "NVIDIA Corporation".to_string(),
                                driver_version: parts[1].trim().to_string(),
                                total_memory_mb: parts[2].trim().split(' ').next()
                                    .map(|s| s.parse::<u64>().unwrap_or(0))
                                    .unwrap_or(0),
                                memory_type: parts[3].trim().to_string(),
                                pcie_speed: format!("{} GT/s", parts[4].trim()),
                                pcie_width: parts[5].trim().parse::<u8>().unwrap_or(16),
                                bios_version: parts.get(6).map(|s| s.trim().to_string()),
                                serial_number: parts.get(7).map(|s| s.trim().to_string()),
                            });
                        }
                    }
                }
            }
        }
        if gpus.is_empty() {
            if let Ok(output) = std::process::Command::new("rocm-smi")
                .args(&[
                    "--showproductname",
                    "--showdriverversion",
                    "--showmeminfo",
                    "vram",
                ])
                .output()
            {
                if output.status.success() {
                    if let Ok(output_str) = String::from_utf8(output.stdout) {
                        let mut name = "Unknown".to_string();
                        let mut driver = "Unknown".to_string();
                        let mut memory = 0;
                        for line in output_str.lines() {
                            if line.contains("GPU") && line.contains("Product Name") {
                                if let Some(n) = line.split(':').nth(1) {
                                    name = n.trim().to_string();
                                }
                            }
                            if line.contains("Driver Version") {
                                if let Some(d) = line.split(':').nth(1) {
                                    driver = d.trim().to_string();
                                }
                            }
                            if line.contains("VRAM") && line.contains("Total") {
                                if let Some(m) = line
                                    .split_whitespace()
                                    .find(|s| s.ends_with("MB"))
                                    .and_then(|s| s.trim_end_matches("MB").parse::<u64>().ok())
                                {
                                    memory = m;
                                }
                            }
                        }
                        if !name.is_empty() && name != "Unknown" {
                            gpus.push(GpuInfo {
                                name,
                                vendor: "AMD".to_string(),
                                driver_version: driver,
                                total_memory_mb: memory,
                                memory_type: "GDDR6".to_string(),
                                pcie_speed: "16 GT/s".to_string(),
                                pcie_width: 16,
                                bios_version: None,
                                serial_number: None,
                            });
                        }
                    }
                }
            }
        }
        if gpus.is_empty() {
            if let Ok(output) = std::process::Command::new("lspci")
                .args(&["-v", "-nn", "-d", "1002:"]) // AMD PCI vendor ID
                .output()
            {
                if output.status.success() {
                    if let Ok(output_str) = String::from_utf8(output.stdout) {
                        for line in output_str.lines() {
                            if line.contains("VGA") || line.contains("Display") {
                                if let Some(name) = line.split('(').next().map(|s| s.trim()) {
                                    gpus.push(GpuInfo {
                                        name: name.to_string(),
                                        vendor: "AMD".to_string(),
                                        driver_version: "Unknown".to_string(),
                                        total_memory_mb: 0,
                                        memory_type: "Unknown".to_string(),
                                        pcie_speed: "Unknown".to_string(),
                                        pcie_width: 16,
                                        bios_version: None,
                                        serial_number: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        if gpus.is_empty() {
            if let Ok(output) = std::process::Command::new("lspci")
                .args(&["-v", "-nn", "-d", "8086:"]) // Intel PCI vendor ID
                .output()
            {
                if output.status.success() {
                    if let Ok(output_str) = String::from_utf8(output.stdout) {
                        for line in output_str.lines() {
                            if line.contains("VGA") || line.contains("Display") {
                                if let Some(name) = line.split('(').next().map(|s| s.trim()) {
                                    gpus.push(GpuInfo {
                                        name: name.to_string(),
                                        vendor: "Intel Corporation".to_string(),
                                        driver_version: "Unknown".to_string(),
                                        total_memory_mb: 0,
                                        memory_type: "Shared".to_string(),
                                        pcie_speed: "Unknown".to_string(),
                                        pcie_width: 16,
                                        bios_version: None,
                                        serial_number: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        if gpus.is_empty() {
            gpus.push(GpuInfo {
                name: "Unknown GPU".to_string(),
                vendor: "Unknown".to_string(),
                driver_version: "Unknown".to_string(),
                total_memory_mb: 0,
                memory_type: "Unknown".to_string(),
                pcie_speed: "Unknown".to_string(),
                pcie_width: 16,
                bios_version: None,
                serial_number: None,
            });
        }

        Ok(gpus)
    }

    #[cfg(target_os = "windows")]
    {
        get_windows_gpus()
    }

    #[cfg(target_os = "macos")]
    {
        let mut gpus = Vec::new();

        if let Ok(output) = std::process::Command::new("system_profiler")
            .args(&["SPDisplaysDataType", "-json"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
                        if let Some(displays) =
                            json.get("SPDisplaysDataType").and_then(|v| v.as_array())
                        {
                            for display in displays {
                                if let (Some(name), Some(vendor)) = (
                                    display.get("sppci_model").and_then(|v| v.as_str()),
                                    display.get("sppci_vendor").and_then(|v| v.as_str()),
                                ) {
                                    let memory = display
                                        .get("spdisplays_vram")
                                        .and_then(|v| v.as_str())
                                        .map(|s| {
                                            let bytes = s.split(' ').next().unwrap_or("0");
                                            bytes.parse::<u64>().unwrap_or(0) / 1024 / 1024
                                        })
                                        .unwrap_or(0);

                                    gpus.push(GpuInfo {
                                        name: name.to_string(),
                                        vendor: vendor.to_string(),
                                        driver_version: display
                                            .get("spdisplays_metalfamily")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("Unknown")
                                            .to_string(),
                                        total_memory_mb: memory,
                                        memory_type: "Unknown".to_string(),
                                        pcie_speed: "Unknown".to_string(),
                                        pcie_width: 16,
                                        bios_version: None,
                                        serial_number: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        if gpus.is_empty() {
            gpus.push(GpuInfo {
                name: "Unknown GPU (macOS)".to_string(),
                vendor: "Unknown".to_string(),
                driver_version: "Unknown".to_string(),
                total_memory_mb: 0,
                memory_type: "Unknown".to_string(),
                pcie_speed: "Unknown".to_string(),
                pcie_width: 16,
                bios_version: None,
                serial_number: None,
            });
        }

        Ok(gpus)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Ok(vec![GpuInfo {
            name: format!("Unknown GPU ({})", std::env::consts::OS),
            vendor: "Unknown".to_string(),
            driver_version: "Unknown".to_string(),
            total_memory_mb: 0,
            memory_type: "Unknown".to_string(),
            pcie_speed: "Unknown".to_string(),
            pcie_width: 16,
            bios_version: None,
            serial_number: None,
        }])
    }
}

#[cfg(target_os = "windows")]
fn get_windows_gpus() -> Result<Vec<GpuInfo>> {
    use std::process::Command;

    let mut gpus = Vec::new();

    let output = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-CimInstance -Namespace root/cimv2 -ClassName Win32_VideoController | Select-Object Name, DriverVersion, AdapterRAM, VideoProcessor, VideoModeDescription"
        ])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                let mut current_gpu = GpuInfo {
                    name: "Unknown".to_string(),
                    vendor: "Unknown".to_string(),
                    driver_version: "Unknown".to_string(),
                    total_memory_mb: 0,
                    memory_type: "Unknown".to_string(),
                    pcie_speed: "Unknown".to_string(),
                    pcie_width: 16,
                    bios_version: None,
                    serial_number: None,
                };
                for line in output_str.lines() {
                    if line.contains(":") {
                        let parts: Vec<&str> = line.split(':').collect();
                        if parts.len() >= 2 {
                            let key = parts[0].trim();
                            let value = parts[1].trim();
                            if key == "Name" {
                                if !current_gpu.name.is_empty() && current_gpu.name != "Unknown" {
                                    gpus.push(current_gpu.clone());
                                }
                                current_gpu.name = value.to_string();
                            } else if key == "DriverVersion" {
                                current_gpu.driver_version = value.to_string();
                            } else if key == "AdapterRAM" {
                                if let Ok(ram) = value.parse::<u64>() {
                                    current_gpu.total_memory_mb = ram / (1024 * 1024);
                                }
                            }
                        }
                    }
                }
                if !current_gpu.name.is_empty() && current_gpu.name != "Unknown" {
                    gpus.push(current_gpu);
                }
            }
        }
    }

    if gpus.is_empty() {
        gpus.push(GpuInfo {
            name: "Unknown GPU (Windows)".to_string(),
            vendor: "Unknown".to_string(),
            driver_version: "Unknown".to_string(),
            total_memory_mb: 0,
            memory_type: "Unknown".to_string(),
            pcie_speed: "Unknown".to_string(),
            pcie_width: 16,
            bios_version: None,
            serial_number: None,
        });
    }

    Ok(gpus)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_info_metadata() {
        let driver = GpuInfoDriver;
        assert_eq!(driver.name(), "gpu_info");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemGpu);
    }
}
