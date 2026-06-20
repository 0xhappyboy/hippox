//! GPU clock driver

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for getting GPU clock speeds
#[derive(Debug)]
pub struct GpuClockDriver;

#[async_trait::async_trait]
impl Driver for GpuClockDriver {
    fn name(&self) -> &str {
        "gpu_clock"
    }

    fn description(&self) -> &str {
        "Get GPU core, memory, and boost clock speeds"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to monitor GPU clock frequencies"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "gpu_clock",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        r#"GPU Clock:
Core: 1350 MHz
Memory: 7000 MHz
Boost: 1500 MHz"#
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
        let clock = get_gpu_clock()?;
        let mut output = String::from("GPU Clock:\n");
        output.push_str(&format!("Core: {} MHz\n", clock.core_mhz));
        output.push_str(&format!("Memory: {} MHz\n", clock.memory_mhz));
        if let Some(boost) = clock.boost_mhz {
            output.push_str(&format!("Boost: {} MHz\n", boost));
        }
        Ok(output)
    }
}

#[derive(Debug, Clone)]
struct GpuClockInfo {
    pub core_mhz: u64,
    pub memory_mhz: u64,
    pub boost_mhz: Option<u64>,
}

fn get_gpu_clock() -> Result<GpuClockInfo> {
    #[cfg(target_os = "linux")]
    {
        let mut core = 0;
        let mut memory = 0;
        let mut boost = None;

        // Try NVIDIA
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(&[
                "--query-gpu",
                "clocks.current.graphics,clocks.current.memory,clocks.max.graphics",
            ])
            .args(&["--format", "csv,noheader"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(line) = output_str.lines().next() {
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() >= 2 {
                            core = parts[0]
                                .trim()
                                .split(' ')
                                .next()
                                .map(|s| s.parse::<u64>().unwrap_or(0))
                                .unwrap_or(0);
                            memory = parts[1]
                                .trim()
                                .split(' ')
                                .next()
                                .map(|s| s.parse::<u64>().unwrap_or(0))
                                .unwrap_or(0);
                            if parts.len() >= 3 {
                                boost = parts[2]
                                    .trim()
                                    .split(' ')
                                    .next()
                                    .map(|s| s.parse::<u64>().unwrap_or(0));
                            }
                        }
                    }
                }
            }
        }

        // Try AMD via rocm-smi
        if core == 0 {
            if let Ok(output) = std::process::Command::new("rocm-smi")
                .args(&["--showclock"])
                .output()
            {
                if output.status.success() {
                    if let Ok(output_str) = String::from_utf8(output.stdout) {
                        for line in output_str.lines() {
                            if line.contains("GPU") && line.contains("MHz") {
                                if line.contains("Core") || line.contains("GFX") {
                                    if let Some(clock) = line
                                        .split_whitespace()
                                        .find(|s| s.contains("MHz"))
                                        .and_then(|s| s.trim_end_matches("MHz").parse::<u64>().ok())
                                    {
                                        core = clock;
                                    }
                                }
                                if line.contains("Memory") || line.contains("MEM") {
                                    if let Some(clock) = line
                                        .split_whitespace()
                                        .find(|s| s.contains("MHz"))
                                        .and_then(|s| s.trim_end_matches("MHz").parse::<u64>().ok())
                                    {
                                        memory = clock;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Try AMD via hwmon
        if core == 0 {
            let hwmon_path = "/sys/class/hwmon";
            if let Ok(entries) = std::fs::read_dir(hwmon_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if let Ok(name) = std::fs::read_to_string(path.join("name")) {
                            if name.trim().contains("amdgpu") {
                                let clock_file = path.join("pp_dpm_sclk");
                                if let Ok(clock_str) = std::fs::read_to_string(&clock_file) {
                                    if let Some(line) = clock_str.lines().last() {
                                        if let Some(clock) = line.split(':').nth(1) {
                                            core = clock
                                                .trim()
                                                .split(' ')
                                                .next()
                                                .map(|s| s.parse::<u64>().unwrap_or(0))
                                                .unwrap_or(0);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(GpuClockInfo {
            core_mhz: core,
            memory_mhz: memory,
            boost_mhz: boost,
        })
    }

    #[cfg(target_os = "windows")]
    {
        get_windows_gpu_clock()
    }

    #[cfg(target_os = "macos")]
    {
        get_macos_gpu_clock()
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Ok(GpuClockInfo {
            core_mhz: 0,
            memory_mhz: 0,
            boost_mhz: None,
        })
    }
}

#[cfg(target_os = "windows")]
fn get_windows_gpu_clock() -> Result<GpuClockInfo> {
    use std::process::Command;

    // Try NVIDIA via nvidia-smi (Windows also has nvidia-smi)
    if let Ok(output) = Command::new("nvidia-smi")
        .args(&[
            "--query-gpu",
            "clocks.current.graphics,clocks.current.memory,clocks.max.graphics",
        ])
        .args(&["--format", "csv,noheader"])
        .output()
    {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                if let Some(line) = output_str.lines().next() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        let core = parts[0]
                            .trim()
                            .split(' ')
                            .next()
                            .map(|s| s.parse::<u64>().unwrap_or(0))
                            .unwrap_or(0);
                        let memory = parts[1]
                            .trim()
                            .split(' ')
                            .next()
                            .map(|s| s.parse::<u64>().unwrap_or(0))
                            .unwrap_or(0);
                        let boost = if parts.len() >= 3 {
                            parts[2]
                                .trim()
                                .split(' ')
                                .next()
                                .map(|s| s.parse::<u64>().unwrap_or(0))
                        } else {
                            None
                        };
                        return Ok(GpuClockInfo {
                            core_mhz: core,
                            memory_mhz: memory,
                            boost_mhz: boost,
                        });
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
                let core = device
                    .clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics)
                    .unwrap_or(0) as u64;
                let memory = device
                    .clock_info(nvml_wrapper::enum_wrappers::device::Clock::Memory)
                    .unwrap_or(0) as u64;
                let boost = device
                    .max_clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics)
                    .unwrap_or(0) as u64;
                return Ok(GpuClockInfo {
                    core_mhz: core,
                    memory_mhz: memory,
                    boost_mhz: Some(boost),
                });
            }
        }
    }

    // Try PowerShell WMI
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-CimInstance -Namespace root/cimv2 -ClassName Win32_VideoController | Select-Object Name, CurrentHorizontalResolution, CurrentVerticalResolution, AdapterRAM"
        ])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(_output_str) = String::from_utf8(output.stdout) {
                // WMI doesn't provide clock speeds directly
                // Return placeholder
            }
        }
    }

    Ok(GpuClockInfo {
        core_mhz: 0,
        memory_mhz: 0,
        boost_mhz: None,
    })
}

#[cfg(target_os = "macos")]
fn get_macos_gpu_clock() -> Result<GpuClockInfo> {
    use std::process::Command;

    // Try system_profiler
    let output = Command::new("system_profiler")
        .args(&["SPDisplaysDataType"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("Chipset Model") || line.contains("Graphics Card") {
                        // macOS system_profiler doesn't provide clock speeds
                        break;
                    }
                }
            }
        }
    }

    // Try IOKit via sysctl
    let output = Command::new("sysctl")
        .args(&["-n", "hw.gpu.core_frequency"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                if let Ok(core) = output_str.trim().parse::<u64>() {
                    return Ok(GpuClockInfo {
                        core_mhz: core,
                        memory_mhz: 0,
                        boost_mhz: None,
                    });
                }
            }
        }
    }

    Ok(GpuClockInfo {
        core_mhz: 0,
        memory_mhz: 0,
        boost_mhz: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_clock_metadata() {
        let driver = GpuClockDriver;
        assert_eq!(driver.name(), "gpu_clock");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemGpu);
    }
}
