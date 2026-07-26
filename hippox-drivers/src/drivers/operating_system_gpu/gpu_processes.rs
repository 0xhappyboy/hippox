//! GPU processes driver
//!
//! This driver provides functionality to list processes currently using the GPU
//! and their memory usage.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing GPU processes
#[derive(Debug)]
pub struct GpuProcessesDriver;
#[async_trait::async_trait]
impl Driver for GpuProcessesDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "gpu_processes"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "List processes currently using the GPU and their memory usage"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to identify which applications are using GPU resources"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "gpu_processes",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"GPU Processes:
PID: 1234 | Process: game.exe | Memory: 2048 MB
PID: 5678 | Process: browser.exe | Memory: 512 MB"#
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
        debug!("Executing gpu_processes driver");
        let processes = get_gpu_processes()?;
        if processes.is_empty() {
            info!("No processes using GPU");
            return Ok("No processes using GPU".to_string());
        }
        info!("Found {} GPU processes", processes.len());
        let mut output = String::from("GPU Processes:\n");
        for proc in processes {
            output.push_str(&format!("PID: {} | Process: {} | Memory: {} MB\n", proc.pid, proc.name, proc.memory_used_mb,));
        }
        return Ok(output);
    }
}
/// Internal GPU process information structure
#[derive(Debug, Clone)]
struct GpuProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory_used_mb: u64,
}
/// Gets GPU processes from the system
fn get_gpu_processes() -> DriverResult<Vec<GpuProcessInfo>> {
    let mut processes: Vec<GpuProcessInfo> = Vec::new();
    #[cfg(target_os = "linux")]
    {
        debug!("Getting GPU processes on Linux");
        // Try NVIDIA
        debug!("Trying NVIDIA nvidia-smi for GPU processes");
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
                                let memory = parts[1].trim().split(' ').next().map(|s| s.parse::<u64>().unwrap_or(0)).unwrap_or(0);
                                processes.push(GpuProcessInfo { pid, name: parts[2].trim().to_string(), memory_used_mb: memory });
                            }
                        }
                    }
                    info!("Found {} NVIDIA GPU processes", processes.len());
                }
            }
        }
        // Try AMD via rocm-smi
        if processes.is_empty() {
            debug!("Trying AMD rocm-smi for GPU processes");
            if let Ok(output) = std::process::Command::new("rocm-smi").args(&["--showpid", "--showprocesses"]).output() {
                if output.status.success() {
                    if let Ok(output_str) = String::from_utf8(output.stdout) {
                        let mut current_pid: Option<u32> = None;
                        let mut current_memory: u64 = 0;
                        let mut current_name = String::new();
                        for line in output_str.lines() {
                            let trimmed = line.trim();
                            if trimmed.contains("GPU") && trimmed.contains("PID") {
                                if let Some(pid_start) = trimmed.find("PID[") {
                                    if let Some(pid_end) = trimmed[pid_start..].find(']') {
                                        if let Ok(pid) = trimmed[pid_start + 4..pid_start + pid_end].parse::<u32>() {
                                            current_pid = Some(pid);
                                        }
                                    }
                                }
                                if let Some(mem_start) = trimmed.find("Memory[") {
                                    if let Some(mem_end) = trimmed[mem_start..].find(']') {
                                        if let Ok(mem) = trimmed[mem_start + 7..mem_start + mem_end].parse::<u64>() {
                                            current_memory = mem;
                                        }
                                    }
                                }
                                if let Some(name_start) = trimmed.find("Name[") {
                                    if let Some(name_end) = trimmed[name_start..].find(']') {
                                        current_name = trimmed[name_start + 5..name_start + name_end].to_string();
                                    }
                                }
                                if let Some(pid) = current_pid {
                                    processes.push(GpuProcessInfo {
                                        pid,
                                        name: if current_name.is_empty() { "Unknown".to_string() } else { current_name.clone() },
                                        memory_used_mb: current_memory,
                                    });
                                    current_pid = None;
                                    current_memory = 0;
                                    current_name.clear();
                                }
                            }
                        }
                        info!("Found {} AMD GPU processes", processes.len());
                    }
                }
            }
        }
        return Ok(processes);
    }
    #[cfg(target_os = "windows")]
    {
        debug!("Getting GPU processes on Windows");
        return get_windows_gpu_processes();
    }
    #[cfg(target_os = "macos")]
    {
        debug!("Getting GPU processes on macOS");
        // macOS: Try using powermetrics
        if let Ok(output) = std::process::Command::new("sudo").args(&["powermetrics", "-n", "1", "--samplers", "gpu_power"]).output() {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("GPU") && line.contains("pid") {
                            if let Some(pid_str) = line.split_whitespace().find(|s| s.contains("pid")) {
                                if let Ok(pid) = pid_str.trim_start_matches("pid=").parse::<u32>() {
                                    processes.push(GpuProcessInfo { pid, name: "Unknown".to_string(), memory_used_mb: 0 });
                                }
                            }
                        }
                    }
                    info!("Found {} macOS GPU processes", processes.len());
                }
            }
        }
        // Also try via system_profiler
        if let Ok(output) = std::process::Command::new("system_profiler").args(&["SPDisplaysDataType"]).output() {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("VRAM") || line.contains("vram") {
                            // macOS system_profiler doesn't provide process-level info
                            debug!("system_profiler does not provide process-level info");
                        }
                    }
                }
            }
        }
        return Ok(processes);
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        debug!("GPU processes not supported on this platform");
        return Ok(processes);
    }
}
/// Gets GPU processes on Windows
#[cfg(target_os = "windows")]
fn get_windows_gpu_processes() -> DriverResult<Vec<GpuProcessInfo>> {
    use std::process::Command;
    let mut processes = Vec::new();
    debug!("Getting GPU processes on Windows via nvidia-smi");
    // Try NVIDIA via nvidia-smi
    if let Ok(output) =
        Command::new("nvidia-smi").args(&["--query-compute-apps", "pid,used_gpu_memory,process_name"]).args(&["--format", "csv,noheader"]).output()
    {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 3 {
                        if let Ok(pid) = parts[0].trim().parse::<u32>() {
                            let memory = parts[1].trim().split(' ').next().map(|s| s.parse::<u64>().unwrap_or(0)).unwrap_or(0);
                            processes.push(GpuProcessInfo { pid, name: parts[2].trim().to_string(), memory_used_mb: memory });
                        }
                    }
                }
                info!("Found {} NVIDIA GPU processes on Windows", processes.len());
            }
        }
    }
    // Try NVML
    #[cfg(feature = "nvml")]
    {
        debug!("Trying NVML for GPU processes on Windows");
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
                    info!("Found {} NVML compute processes", processes.len());
                }
                if let Ok(procs) = device.running_graphics_processes() {
                    for proc in procs {
                        processes.push(GpuProcessInfo {
                            pid: proc.pid,
                            name: "Unknown".to_string(),
                            memory_used_mb: proc.used_gpu_memory / (1024 * 1024),
                        });
                    }
                    info!("Found {} NVML graphics processes", processes.len());
                }
            }
        }
    }
    // Try PowerShell WMI
    debug!("Trying PowerShell WMI for GPU processes on Windows");
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
                        debug!("WMI does not provide PID-level info");
                    }
                }
            }
        }
    }
    return Ok(processes);
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
