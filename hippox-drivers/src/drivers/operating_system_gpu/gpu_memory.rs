//! GPU memory driver
//!
//! This driver provides functionality to get GPU memory usage including total, used, and free.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting GPU memory usage
#[derive(Debug)]
pub struct GpuMemoryDriver;
#[async_trait::async_trait]
impl Driver for GpuMemoryDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "gpu_memory"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get GPU memory usage including total, used, and free"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to monitor GPU memory consumption"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "gpu_memory",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"GPU Memory:
Total: 10240 MB
Used: 4520 MB
Free: 5720 MB
Usage: 44.1%"#
            .to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemGpu;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing gpu_memory driver");
        let memory = get_gpu_memory()?;
        info!("GPU memory: total={}MB, used={}MB, free={}MB", memory.total_mb, memory.used_mb, memory.free_mb);
        let output = format!(
            "GPU Memory:\n\
             Total: {} MB\n\
             Used: {} MB\n\
             Free: {} MB\n\
             Usage: {:.1}%",
            memory.total_mb, memory.used_mb, memory.free_mb, memory.usage_percent
        );
        return Ok(output);
    }
}
/// Internal GPU memory information structure
#[derive(Debug, Clone)]
struct GpuMemoryInfo {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub usage_percent: f32,
}
/// Gets GPU memory information from the system
fn get_gpu_memory() -> DriverResult<GpuMemoryInfo> {
    #[cfg(target_os = "linux")]
    {
        debug!("Getting GPU memory on Linux");
        // Try NVIDIA
        debug!("Trying NVIDIA nvidia-smi for memory info");
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(&["--query-gpu", "memory.total,memory.used,memory.free"])
            .args(&["--format", "csv,noheader"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(line) = output_str.lines().next() {
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() >= 3 {
                            let total = parts[0].trim().split(' ').next().map(|s| s.parse::<u64>().unwrap_or(0)).unwrap_or(0);
                            let used = parts[1].trim().split(' ').next().map(|s| s.parse::<u64>().unwrap_or(0)).unwrap_or(0);
                            let free = parts[2].trim().split(' ').next().map(|s| s.parse::<u64>().unwrap_or(0)).unwrap_or(0);
                            info!("NVIDIA memory: total={}MB, used={}MB, free={}MB", total, used, free);
                            return Ok(GpuMemoryInfo {
                                total_mb: total,
                                used_mb: used,
                                free_mb: free,
                                usage_percent: if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 },
                            });
                        }
                    }
                }
            }
        }
        // Try AMD GPU
        debug!("Trying AMD rocm-smi for memory info");
        if let Ok(output) = std::process::Command::new("rocm-smi").args(&["--showmeminfo", "vram"]).output() {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    let mut total = 0;
                    let mut used = 0;
                    for line in output_str.lines() {
                        if line.contains("VRAM") {
                            if line.contains("Total") {
                                if let Some(val) = line.split_whitespace().find(|s| s.ends_with("MB")) {
                                    total = val.trim_end_matches("MB").parse::<u64>().unwrap_or(0);
                                }
                            } else if line.contains("Used") || line.contains("Active") {
                                if let Some(val) = line.split_whitespace().find(|s| s.ends_with("MB")) {
                                    used = val.trim_end_matches("MB").parse::<u64>().unwrap_or(0);
                                }
                            }
                        }
                    }
                    if total > 0 {
                        info!("AMD rocm-smi memory: total={}MB, used={}MB", total, used);
                        return Ok(GpuMemoryInfo {
                            total_mb: total,
                            used_mb: used,
                            free_mb: total - used,
                            usage_percent: (used as f32 / total as f32) * 100.0,
                        });
                    }
                }
            }
        }
        // Try AMD via hwmon
        debug!("Trying AMD hwmon for memory info");
        let hwmon_path = "/sys/class/hwmon";
        if let Ok(entries) = std::fs::read_dir(hwmon_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Ok(name) = std::fs::read_to_string(path.join("name")) {
                        if name.trim().contains("amdgpu") {
                            let mem_info_path = path.join("device/mem_info_vram_total");
                            let mem_used_path = path.join("device/mem_info_vram_used");
                            if let (Ok(total_str), Ok(used_str)) = (std::fs::read_to_string(&mem_info_path), std::fs::read_to_string(&mem_used_path))
                            {
                                let total = total_str.trim().parse::<u64>().unwrap_or(0) / (1024 * 1024);
                                let used = used_str.trim().parse::<u64>().unwrap_or(0) / (1024 * 1024);
                                if total > 0 {
                                    info!("AMD hwmon memory: total={}MB, used={}MB", total, used);
                                    return Ok(GpuMemoryInfo {
                                        total_mb: total,
                                        used_mb: used,
                                        free_mb: total - used,
                                        usage_percent: (used as f32 / total as f32) * 100.0,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        info!("GPU memory not found on Linux");
        return Ok(GpuMemoryInfo { total_mb: 0, used_mb: 0, free_mb: 0, usage_percent: 0.0 });
    }
    #[cfg(target_os = "windows")]
    {
        debug!("Getting GPU memory on Windows");
        return get_windows_gpu_memory();
    }
    #[cfg(target_os = "macos")]
    {
        debug!("GPU memory not available on macOS");
        return Ok(GpuMemoryInfo { total_mb: 0, used_mb: 0, free_mb: 0, usage_percent: 0.0 });
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        debug!("GPU memory not supported on this platform");
        return Ok(GpuMemoryInfo { total_mb: 0, used_mb: 0, free_mb: 0, usage_percent: 0.0 });
    }
}
/// Gets GPU memory on Windows
#[cfg(target_os = "windows")]
fn get_windows_gpu_memory() -> DriverResult<GpuMemoryInfo> {
    use std::process::Command;
    debug!("Getting GPU memory on Windows via PowerShell WMI");
    let output = Command::new("powershell")
        .args(&["-Command", "Get-CimInstance -Namespace root/cimv2 -ClassName Win32_VideoController | Select-Object Name, AdapterRAM"])
        .output();
    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("MB") || line.contains("GB") {
                        if let Some(size_str) = line.split_whitespace().find(|s| s.contains("MB") || s.contains("GB")) {
                            let total = if size_str.contains("GB") {
                                size_str.trim_end_matches("GB").parse::<u64>().unwrap_or(0) * 1024
                            } else {
                                size_str.trim_end_matches("MB").parse::<u64>().unwrap_or(0)
                            };
                            if total > 0 {
                                info!("Windows GPU memory total: {}MB", total);
                                return Ok(GpuMemoryInfo { total_mb: total, used_mb: 0, free_mb: total, usage_percent: 0.0 });
                            }
                        }
                    }
                }
            }
        }
    }
    info!("GPU memory not found on Windows");
    return Ok(GpuMemoryInfo { total_mb: 0, used_mb: 0, free_mb: 0, usage_percent: 0.0 });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gpu_memory_metadata() {
        let driver = GpuMemoryDriver;
        assert_eq!(driver.name(), "gpu_memory");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemGpu);
    }
}
