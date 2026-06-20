//! GPU memory driver

use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for getting GPU memory usage
#[derive(Debug)]
pub struct GpuMemoryDriver;

#[async_trait::async_trait]
impl Driver for GpuMemoryDriver {
    fn name(&self) -> &str {
        "gpu_memory"
    }

    fn description(&self) -> &str {
        "Get GPU memory usage including total, used, and free"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to monitor GPU memory consumption"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "gpu_memory",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        r#"GPU Memory:
Total: 10240 MB
Used: 4520 MB
Free: 5720 MB
Usage: 44.1%"#
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
        let memory = get_gpu_memory()?;

        let output = format!(
            "GPU Memory:\n\
             Total: {} MB\n\
             Used: {} MB\n\
             Free: {} MB\n\
             Usage: {:.1}%",
            memory.total_mb, memory.used_mb, memory.free_mb, memory.usage_percent
        );
        Ok(output)
    }
}

#[derive(Debug, Clone)]
struct GpuMemoryInfo {
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub usage_percent: f32,
}

fn get_gpu_memory() -> Result<GpuMemoryInfo> {
    #[cfg(target_os = "linux")]
    {
        // Try NVIDIA
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
                            let total = parts[0]
                                .trim()
                                .split(' ')
                                .next()
                                .map(|s| s.parse::<u64>().unwrap_or(0))
                                .unwrap_or(0);
                            let used = parts[1]
                                .trim()
                                .split(' ')
                                .next()
                                .map(|s| s.parse::<u64>().unwrap_or(0))
                                .unwrap_or(0);
                            let free = parts[2]
                                .trim()
                                .split(' ')
                                .next()
                                .map(|s| s.parse::<u64>().unwrap_or(0))
                                .unwrap_or(0);

                            return Ok(GpuMemoryInfo {
                                total_mb: total,
                                used_mb: used,
                                free_mb: free,
                                usage_percent: if total > 0 {
                                    (used as f32 / total as f32) * 100.0
                                } else {
                                    0.0
                                },
                            });
                        }
                    }
                }
            }
        }
        // Try AMD GPU
        if let Ok(output) = std::process::Command::new("rocm-smi")
            .args(&["--showmeminfo", "vram"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    let mut total = 0;
                    let mut used = 0;
                    for line in output_str.lines() {
                        if line.contains("VRAM") {
                            if line.contains("Total") {
                                if let Some(val) =
                                    line.split_whitespace().find(|s| s.ends_with("MB"))
                                {
                                    total = val.trim_end_matches("MB").parse::<u64>().unwrap_or(0);
                                }
                            } else if line.contains("Used") || line.contains("Active") {
                                if let Some(val) =
                                    line.split_whitespace().find(|s| s.ends_with("MB"))
                                {
                                    used = val.trim_end_matches("MB").parse::<u64>().unwrap_or(0);
                                }
                            }
                        }
                    }
                    if total > 0 {
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
        let hwmon_path = "/sys/class/hwmon";
        if let Ok(entries) = std::fs::read_dir(hwmon_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Ok(name) = std::fs::read_to_string(path.join("name")) {
                        if name.trim().contains("amdgpu") {
                            let mem_info_path = path.join("device/mem_info_vram_total");
                            let mem_used_path = path.join("device/mem_info_vram_used");
                            if let (Ok(total_str), Ok(used_str)) = (
                                std::fs::read_to_string(&mem_info_path),
                                std::fs::read_to_string(&mem_used_path),
                            ) {
                                let total =
                                    total_str.trim().parse::<u64>().unwrap_or(0) / (1024 * 1024);
                                let used =
                                    used_str.trim().parse::<u64>().unwrap_or(0) / (1024 * 1024);
                                if total > 0 {
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

        Ok(GpuMemoryInfo {
            total_mb: 0,
            used_mb: 0,
            free_mb: 0,
            usage_percent: 0.0,
        })
    }

    #[cfg(target_os = "windows")]
    {
        get_windows_gpu_memory()
    }

    #[cfg(target_os = "macos")]
    {
        Ok(GpuMemoryInfo {
            total_mb: 0,
            used_mb: 0,
            free_mb: 0,
            usage_percent: 0.0,
        })
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Ok(GpuMemoryInfo {
            total_mb: 0,
            used_mb: 0,
            free_mb: 0,
            usage_percent: 0.0,
        })
    }
}

#[cfg(target_os = "windows")]
fn get_windows_gpu_memory() -> Result<GpuMemoryInfo> {
    use std::process::Command;

    let output = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-CimInstance -Namespace root/cimv2 -ClassName Win32_VideoController | Select-Object Name, AdapterRAM"
        ])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("MB") || line.contains("GB") {
                        if let Some(size_str) = line
                            .split_whitespace()
                            .find(|s| s.contains("MB") || s.contains("GB"))
                        {
                            let total = if size_str.contains("GB") {
                                size_str.trim_end_matches("GB").parse::<u64>().unwrap_or(0) * 1024
                            } else {
                                size_str.trim_end_matches("MB").parse::<u64>().unwrap_or(0)
                            };
                            if total > 0 {
                                return Ok(GpuMemoryInfo {
                                    total_mb: total,
                                    used_mb: 0,
                                    free_mb: total,
                                    usage_percent: 0.0,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(GpuMemoryInfo {
        total_mb: 0,
        used_mb: 0,
        free_mb: 0,
        usage_percent: 0.0,
    })
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
