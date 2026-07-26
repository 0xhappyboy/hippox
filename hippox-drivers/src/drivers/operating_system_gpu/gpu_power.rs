//! GPU power driver
//!
//! This driver provides functionality to get GPU power consumption in watts.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting GPU power consumption
#[derive(Debug)]
pub struct GpuPowerDriver;
#[async_trait::async_trait]
impl Driver for GpuPowerDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "gpu_power"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get GPU power consumption in watts"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to monitor GPU power usage"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "gpu_power",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "GPU Power: 175.5 W".to_string();
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
        debug!("Executing gpu_power driver");
        let power = get_gpu_power()?;
        info!("GPU power: {:.1} W", power);
        return Ok(format!("GPU Power: {:.1} W", power));
    }
}
/// Gets GPU power consumption from the system
fn get_gpu_power() -> DriverResult<f32> {
    #[cfg(target_os = "linux")]
    {
        debug!("Getting GPU power on Linux");
        // Try NVIDIA
        debug!("Trying NVIDIA nvidia-smi for power info");
        if let Ok(output) = std::process::Command::new("nvidia-smi").args(&["--query-gpu", "power.draw"]).args(&["--format", "csv,noheader"]).output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(line) = output_str.lines().next() {
                        if let Ok(power) = line.trim().split(' ').next().map(|s| s.parse::<f32>()).unwrap_or(Ok(0.0)) {
                            info!("NVIDIA power: {:.1} W", power);
                            return Ok(power);
                        }
                    }
                }
            }
        }
        // Try AMD via rocm-smi
        debug!("Trying AMD rocm-smi for power info");
        if let Ok(output) = std::process::Command::new("rocm-smi").args(&["--showpower"]).output() {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("GPU") && line.contains("W") {
                            if let Some(power_str) = line.split_whitespace().find(|s| s.contains("W") && !s.contains("Limit")) {
                                if let Ok(power) = power_str.trim_end_matches("W").parse::<f32>() {
                                    info!("AMD rocm-smi power: {:.1} W", power);
                                    return Ok(power);
                                }
                            }
                        }
                    }
                }
            }
        }
        // Try AMD via hwmon
        debug!("Trying AMD hwmon for power info");
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
                                    let watts = power / 1_000_000.0;
                                    info!("AMD hwmon power: {:.1} W", watts);
                                    return Ok(watts);
                                }
                            }
                        }
                    }
                }
            }
        }
        info!("GPU power not found on Linux");
        return Ok(0.0);
    }
    #[cfg(target_os = "windows")]
    {
        debug!("Getting GPU power on Windows");
        use std::process::Command;
        let output = Command::new("powershell")
            .args(&["-Command", "Get-CimInstance -Namespace root/cimv2 -ClassName Win32_VideoController | Select-Object Name, AdapterRAM"])
            .output();
        #[cfg(feature = "nvml")]
        {
            debug!("Trying NVML for power info on Windows");
            use nvml_wrapper::Nvml;
            if let Ok(nvml) = Nvml::init() {
                if let Ok(device) = nvml.device_by_index(0) {
                    if let Ok(power) = device.power_usage() {
                        let watts = power as f32 / 1000.0;
                        info!("Windows NVML power: {:.1} W", watts);
                        return Ok(watts);
                    }
                }
            }
        }
        info!("GPU power not found on Windows");
        return Ok(0.0);
    }
    #[cfg(target_os = "macos")]
    {
        debug!("Getting GPU power on macOS");
        if let Ok(output) = std::process::Command::new("sudo").args(&["powermetrics", "-n", "1", "--samplers", "gpu_power"]).output() {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("GPU Power") {
                            if let Some(power_str) = line.split(':').nth(1) {
                                if let Ok(power) = power_str.trim().trim_end_matches("mW").parse::<f32>() {
                                    let watts = power / 1000.0;
                                    info!("macOS power: {:.1} W", watts);
                                    return Ok(watts);
                                }
                            }
                        }
                    }
                }
            }
        }
        info!("GPU power not found on macOS");
        return Ok(0.0);
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        debug!("GPU power not supported on this platform");
        return Ok(0.0);
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
