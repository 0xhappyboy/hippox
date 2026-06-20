//! GPU usage driver

use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for getting GPU usage
#[derive(Debug)]
pub struct GpuUsageDriver;

#[async_trait::async_trait]
impl Driver for GpuUsageDriver {
    fn name(&self) -> &str {
        "gpu_usage"
    }

    fn description(&self) -> &str {
        "Get current GPU usage percentage"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to monitor GPU utilization for performance analysis"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "gpu_usage",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        "GPU Usage: 67.5%".to_string()
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
        let usage = get_gpu_usage()?;
        Ok(format!("GPU Usage: {:.1}%", usage))
    }
}

fn get_gpu_usage() -> Result<f32> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(&["--query-gpu", "utilization.gpu"])
            .args(&["--format", "csv,noheader"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(line) = output_str.lines().next() {
                        if let Ok(usage) = line.trim().trim_end_matches('%').parse::<f32>() {
                            return Ok(usage);
                        }
                    }
                }
            }
        }
        if let Ok(output) = std::process::Command::new("radeontop")
            .args(&["--dump", "1"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.starts_with("gpu") {
                            if let Some(percent) = line.split_whitespace().nth(1) {
                                if let Ok(usage) = percent.trim_end_matches('%').parse::<f32>() {
                                    return Ok(usage);
                                }
                            }
                        }
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
                        let name_str = name.trim();
                        if name_str.contains("amdgpu") || name_str.contains("radeon") {
                            let load_path = path.join("device/gpu_busy_percent");
                            if let Ok(load_str) = std::fs::read_to_string(&load_path) {
                                if let Ok(usage) = load_str.trim().parse::<f32>() {
                                    return Ok(usage);
                                }
                            }
                        }
                    }
                }
            }
        }
        if let Ok(output) = std::process::Command::new("rocm-smi")
            .args(&["--showuse"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("GPU") && line.contains("%") {
                            if let Some(usage) = line
                                .split_whitespace()
                                .find(|s| s.contains("%"))
                                .and_then(|s| s.trim_end_matches('%').parse::<f32>().ok())
                            {
                                return Ok(usage);
                            }
                        }
                    }
                }
            }
        }

        Ok(0.0)
    }

    #[cfg(target_os = "windows")]
    {
        get_windows_gpu_usage()
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("sudo")
            .args(&["powermetrics", "-n", "1", "--samplers", "gpu_power"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("GPU active residency") {
                            if let Some(percent) = line.split(':').nth(1) {
                                if let Ok(usage) =
                                    percent.trim().trim_end_matches('%').parse::<f32>()
                                {
                                    return Ok(usage);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(0.0)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Ok(0.0)
    }
}

#[cfg(target_os = "windows")]
fn get_windows_gpu_usage() -> Result<f32> {
    use std::process::Command;
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-CimInstance -Namespace root/cimv2 -ClassName Win32_PerfFormattedData_GPUPerformanceCounters | Select-Object -ExpandProperty GPUUsage"
        ])
        .output();
    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        if let Ok(usage) = trimmed.parse::<f32>() {
                            if usage > 0.0 && usage <= 100.0 {
                                return Ok(usage);
                            }
                        }
                    }
                }
            }
        }
    }
    #[cfg(feature = "nvml")]
    {
        use nvml_wrapper::Nvml;
        if let Ok(nvml) = Nvml::init() {
            if let Ok(device) = nvml.device_by_index(0) {
                if let Ok(utilization) = device.utilization_rates() {
                    return Ok(utilization.gpu as f32);
                }
            }
        }
    }

    Ok(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_usage_metadata() {
        let driver = GpuUsageDriver;
        assert_eq!(driver.name(), "gpu_usage");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemGpu);
    }
}
