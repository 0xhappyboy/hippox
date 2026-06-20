//! GPU power driver

use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for getting GPU power consumption
#[derive(Debug)]
pub struct GpuPowerDriver;

#[async_trait::async_trait]
impl Driver for GpuPowerDriver {
    fn name(&self) -> &str {
        "gpu_power"
    }

    fn description(&self) -> &str {
        "Get GPU power consumption in watts"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to monitor GPU power usage"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "gpu_power",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        "GPU Power: 175.5 W".to_string()
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
        let power = get_gpu_power()?;
        Ok(format!("GPU Power: {:.1} W", power))
    }
}

fn get_gpu_power() -> Result<f32> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(&["--query-gpu", "power.draw"])
            .args(&["--format", "csv,noheader"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(line) = output_str.lines().next() {
                        if let Ok(power) = line
                            .trim()
                            .split(' ')
                            .next()
                            .map(|s| s.parse::<f32>())
                            .unwrap_or(Ok(0.0))
                        {
                            return Ok(power);
                        }
                    }
                }
            }
        }
        if let Ok(output) = std::process::Command::new("rocm-smi")
            .args(&["--showpower"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("GPU") && line.contains("W") {
                            if let Some(power_str) = line
                                .split_whitespace()
                                .find(|s| s.contains("W") && !s.contains("Limit"))
                            {
                                if let Ok(power) = power_str.trim_end_matches("W").parse::<f32>() {
                                    return Ok(power);
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
                        if name.trim().contains("amdgpu") {
                            let power_file = path.join("power1_input");
                            if let Ok(power_str) = std::fs::read_to_string(&power_file) {
                                if let Ok(power) = power_str.trim().parse::<f32>() {
                                    // power is in microwatts
                                    return Ok(power / 1_000_000.0);
                                }
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
        use std::process::Command;
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                "Get-CimInstance -Namespace root/cimv2 -ClassName Win32_VideoController | Select-Object Name, AdapterRAM"
            ])
            .output();
        #[cfg(feature = "nvml")]
        {
            use nvml_wrapper::Nvml;
            if let Ok(nvml) = Nvml::init() {
                if let Ok(device) = nvml.device_by_index(0) {
                    if let Ok(power) = device.power_usage() {
                        return Ok(power as f32 / 1000.0);
                    }
                }
            }
        }
        Ok(0.0)
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
                        if line.contains("GPU Power") {
                            if let Some(power_str) = line.split(':').nth(1) {
                                if let Ok(power) =
                                    power_str.trim().trim_end_matches("mW").parse::<f32>()
                                {
                                    return Ok(power / 1000.0);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_power_metadata() {
        let driver = GpuPowerDriver;
        assert_eq!(driver.name(), "gpu_power");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemGpu);
    }
}
