//! GPU processes driver

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for listing GPU processes
#[derive(Debug)]
pub struct GpuProcessesDriver;

#[async_trait::async_trait]
impl Driver for GpuProcessesDriver {
    fn name(&self) -> &str {
        "gpu_processes"
    }

    fn description(&self) -> &str {
        "List processes currently using the GPU and their memory usage"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to identify which applications are using GPU resources"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "gpu_processes",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        r#"GPU Processes:
PID: 1234 | Process: game.exe | Memory: 2048 MB
PID: 5678 | Process: browser.exe | Memory: 512 MB"#
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
        let processes = get_gpu_processes()?;
        if processes.is_empty() {
            return Ok("No processes using GPU".to_string());
        }
        let mut output = String::from("GPU Processes:\n");
        for proc in processes {
            output.push_str(&format!(
                "PID: {} | Process: {} | Memory: {} MB\n",
                proc.pid, proc.name, proc.memory_used_mb,
            ));
        }
        Ok(output)
    }
}

#[derive(Debug, Clone)]
struct GpuProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory_used_mb: u64,
}

fn get_gpu_processes() -> Result<Vec<GpuProcessInfo>> {
    let mut processes: Vec<GpuProcessInfo> = Vec::new();

    #[cfg(target_os = "linux")]
    {
        // Try NVIDIA
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(&["--query-compute-apps", "pid,used_gpu_memory,process_name"])
            .args(&["--format", "csv,noheader"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() >= 3 {
                            if let Ok(pid) = parts[0].trim().parse::<u32>() {
                                let memory = parts[1]
                                    .trim()
                                    .split(' ')
                                    .next()
                                    .map(|s| s.parse::<u64>().unwrap_or(0))
                                    .unwrap_or(0);
                                processes.push(GpuProcessInfo {
                                    pid,
                                    name: parts[2].trim().to_string(),
                                    memory_used_mb: memory,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Try AMD via rocm-smi
        if let Ok(output) = std::process::Command::new("rocm-smi")
            .args(&["--showpid", "--showprocesses"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    let mut current_pid: Option<u32> = None;
                    let mut current_memory: u64 = 0;
                    let mut current_name = String::new();

                    for line in output_str.lines() {
                        let trimmed = line.trim();
                        if trimmed.contains("GPU") && trimmed.contains("PID") {
                            // Parse line like: "GPU[0] PID[1234] Name[process] Memory[1024]"
                            if let Some(pid_start) = trimmed.find("PID[") {
                                if let Some(pid_end) = trimmed[pid_start..].find(']') {
                                    if let Ok(pid) =
                                        trimmed[pid_start + 4..pid_start + pid_end].parse::<u32>()
                                    {
                                        current_pid = Some(pid);
                                    }
                                }
                            }
                            if let Some(mem_start) = trimmed.find("Memory[") {
                                if let Some(mem_end) = trimmed[mem_start..].find(']') {
                                    if let Ok(mem) =
                                        trimmed[mem_start + 7..mem_start + mem_end].parse::<u64>()
                                    {
                                        current_memory = mem;
                                    }
                                }
                            }
                            if let Some(name_start) = trimmed.find("Name[") {
                                if let Some(name_end) = trimmed[name_start..].find(']') {
                                    current_name =
                                        trimmed[name_start + 5..name_start + name_end].to_string();
                                }
                            }
                            if let Some(pid) = current_pid {
                                processes.push(GpuProcessInfo {
                                    pid,
                                    name: if current_name.is_empty() {
                                        "Unknown".to_string()
                                    } else {
                                        current_name.clone()
                                    },
                                    memory_used_mb: current_memory,
                                });
                                current_pid = None;
                                current_memory = 0;
                                current_name.clear();
                            }
                        }
                    }
                }
            }
        }
        Ok(processes)
    }

    #[cfg(target_os = "windows")]
    {
        get_windows_gpu_processes()
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: Try using powermetrics
        if let Ok(output) = std::process::Command::new("sudo")
            .args(&["powermetrics", "-n", "1", "--samplers", "gpu_power"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("GPU") && line.contains("pid") {
                            if let Some(pid_str) =
                                line.split_whitespace().find(|s| s.contains("pid"))
                            {
                                if let Ok(pid) = pid_str.trim_start_matches("pid=").parse::<u32>() {
                                    processes.push(GpuProcessInfo {
                                        pid,
                                        name: "Unknown".to_string(),
                                        memory_used_mb: 0,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        // Also try via system_profiler
        if let Ok(output) = std::process::Command::new("system_profiler")
            .args(&["SPDisplaysDataType"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("VRAM") || line.contains("vram") {
                            // macOS system_profiler doesn't provide process-level info
                        }
                    }
                }
            }
        }
        Ok(processes)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Ok(processes)
    }
}

#[cfg(target_os = "windows")]
fn get_windows_gpu_processes() -> Result<Vec<GpuProcessInfo>> {
    use std::process::Command;
    let mut processes = Vec::new();

    // Try NVIDIA via nvidia-smi
    if let Ok(output) = Command::new("nvidia-smi")
        .args(&["--query-compute-apps", "pid,used_gpu_memory,process_name"])
        .args(&["--format", "csv,noheader"])
        .output()
    {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 3 {
                        if let Ok(pid) = parts[0].trim().parse::<u32>() {
                            let memory = parts[1]
                                .trim()
                                .split(' ')
                                .next()
                                .map(|s| s.parse::<u64>().unwrap_or(0))
                                .unwrap_or(0);
                            processes.push(GpuProcessInfo {
                                pid,
                                name: parts[2].trim().to_string(),
                                memory_used_mb: memory,
                            });
                        }
                    }
                }
            }
        }
    }

    // Try NVML
    #[cfg(feature = "nvml")]
    {
        use nvml_wrapper::Nvml;
        if let Ok(nvml) = Nvml::init() {
            if let Ok(device) = nvml.device_by_index(0) {
                if let Ok(procs) = device.running_compute_processes() {
                    for proc in procs {
                        processes.push(GpuProcessInfo {
                            pid: proc.pid,
                            name: "Unknown".to_string(),
                            memory_used_mb: proc.used_gpu_memory / (1024 * 1024),
                        });
                    }
                }
                if let Ok(procs) = device.running_graphics_processes() {
                    for proc in procs {
                        processes.push(GpuProcessInfo {
                            pid: proc.pid,
                            name: "Unknown".to_string(),
                            memory_used_mb: proc.used_gpu_memory / (1024 * 1024),
                        });
                    }
                }
            }
        }
    }

    // Try PowerShell WMI
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-CimInstance -Namespace root/cimv2 -ClassName Win32_PerfFormattedData_GPUPerformanceCounters | Select-Object Name, GPUUsage, GPUAvailableMemory, GPUCommittedMemory"
        ])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("GPU") && line.contains("committed") {
                        // WMI doesn't provide PID-level info in this class
                    }
                }
            }
        }
    }

    Ok(processes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_processes_metadata() {
        let driver = GpuProcessesDriver;
        assert_eq!(driver.name(), "gpu_processes");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemGpu);
    }
}
