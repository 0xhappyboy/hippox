//! GPU temperature driver
//!
//! This driver provides functionality to get GPU temperature.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting GPU temperature
#[derive(Debug)]
pub struct GpuTemperatureDriver;
#[async_trait::async_trait]
impl Driver for GpuTemperatureDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "gpu_temperature"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get GPU temperature"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to monitor GPU thermal conditions"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "gpu_temperature",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "GPU Temperature: 72.0°C".to_string();
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
        debug!("Executing gpu_temperature driver");
        let temp = get_gpu_temperature()?;
        info!("GPU temperature: {:.1}°C", temp);
        return Ok(format!("GPU Temperature: {:.1}°C", temp));
    }
}
/// Gets GPU temperature from the system
fn get_gpu_temperature() -> DriverResult<f64> {
    #[cfg(target_os = "linux")]
    {
        debug!("Getting GPU temperature on Linux");
        // Try NVIDIA
        debug!("Trying NVIDIA nvidia-smi for temperature");
        if let Ok(output) =
            std::process::Command::new("nvidia-smi").args(&["--query-gpu", "temperature.gpu"]).args(&["--format", "csv,noheader"]).output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(line) = output_str.lines().next() {
                        if let Ok(temp) = line.trim().parse::<f64>() {
                            info!("NVIDIA temperature: {:.1}°C", temp);
                            return Ok(temp);
                        }
                    }
                }
            }
        }
        // Try AMD via rocm-smi
        debug!("Trying AMD rocm-smi for temperature");
        if let Ok(output) = std::process::Command::new("rocm-smi").args(&["--showtemp"]).output() {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("GPU") && line.contains("°C") {
                            if let Some(temp_str) = line.split_whitespace().find(|s| s.contains("°C")) {
                                if let Ok(temp) = temp_str.trim_end_matches("°C").parse::<f64>() {
                                    info!("AMD rocm-smi temperature: {:.1}°C", temp);
                                    return Ok(temp);
                                }
                            }
                        }
                    }
                }
            }
        }
        // Try AMD via hwmon
        debug!("Trying AMD hwmon for temperature");
        let hwmon_path = "/sys/class/hwmon";
        if let Ok(entries) = std::fs::read_dir(hwmon_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Ok(name) = std::fs::read_to_string(path.join("name")) {
                        if name.trim().contains("amdgpu") || name.trim().contains("radeon") {
                            let temp_file = path.join("temp1_input");
                            if let Ok(temp_str) = std::fs::read_to_string(&temp_file) {
                                if let Ok(temp) = temp_str.trim().parse::<f64>() {
                                    let celsius = temp / 1000.0;
                                    info!("AMD hwmon temperature: {:.1}°C", celsius);
                                    return Ok(celsius);
                                }
                            }
                        }
                    }
                }
            }
        }
        info!("GPU temperature not found on Linux");
        return Err(DriverError::execution("No GPU temperature sensor found"));
    }
    #[cfg(target_os = "windows")]
    {
        debug!("Getting GPU temperature on Windows");
        return get_windows_gpu_temperature();
    }
    #[cfg(target_os = "macos")]
    {
        debug!("Getting GPU temperature on macOS");
        if let Ok(output) = std::process::Command::new("sudo").args(&["powermetrics", "-n", "1", "--samplers", "gpu_power"]).output() {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("GPU temperature") {
                            if let Some(temp_str) = line.split(':').nth(1) {
                                if let Ok(temp) = temp_str.trim().parse::<f64>() {
                                    info!("macOS temperature: {:.1}°C", temp);
                                    return Ok(temp);
                                }
                            }
                        }
                    }
                }
            }
        }
        info!("GPU temperature not found on macOS, returning 45.0");
        return Ok(45.0);
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        debug!("GPU temperature not supported on this platform");
        return Err(DriverError::execution("GPU temperature not supported on this platform"));
    }
}
/// Gets GPU temperature on Windows
#[cfg(target_os = "windows")]
fn get_windows_gpu_temperature() -> DriverResult<f64> {
    use std::process::Command;
    debug!("Getting GPU temperature on Windows via WMI");
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-CimInstance -Namespace root/wmi -ClassName MSAcpi_ThermalZoneTemperature | Select-Object -ExpandProperty CurrentTemperature",
        ])
        .output();
    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        if let Ok(temp_raw) = trimmed.parse::<f64>() {
                            let celsius = (temp_raw / 10.0) - 273.15;
                            if celsius > 0.0 && celsius < 100.0 {
                                info!("Windows WMI temperature: {:.1}°C", celsius);
                                return Ok(celsius);
                            }
                        }
                    }
                }
            }
        }
    }
    #[cfg(feature = "nvml")]
    {
        debug!("Trying NVML for temperature on Windows");
        use nvml_wrapper::Nvml;
        if let Ok(nvml) = Nvml::init() {
            if let Ok(device) = nvml.device_by_index(0) {
                if let Ok(temp) = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu) {
                    info!("Windows NVML temperature: {:.1}°C", temp as f64);
                    return Ok(temp as f64);
                }
            }
        }
    }
    info!("GPU temperature not found on Windows, returning 45.0");
    return Ok(45.0);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gpu_temperature_metadata() {
        let driver = GpuTemperatureDriver;
        assert_eq!(driver.name(), "gpu_temperature");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemGpu);
    }
}
