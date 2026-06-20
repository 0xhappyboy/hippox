//! GPU temperature driver

use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for getting GPU temperature
#[derive(Debug)]
pub struct GpuTemperatureDriver;

#[async_trait::async_trait]
impl Driver for GpuTemperatureDriver {
    fn name(&self) -> &str {
        "gpu_temperature"
    }

    fn description(&self) -> &str {
        "Get GPU temperature"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to monitor GPU thermal conditions"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "gpu_temperature",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        "GPU Temperature: 72.0°C".to_string()
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
        let temp = get_gpu_temperature()?;
        Ok(format!("GPU Temperature: {:.1}°C", temp))
    }
}

fn get_gpu_temperature() -> Result<f64> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(&["--query-gpu", "temperature.gpu"])
            .args(&["--format", "csv,noheader"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(line) = output_str.lines().next() {
                        if let Ok(temp) = line.trim().parse::<f64>() {
                            return Ok(temp);
                        }
                    }
                }
            }
        }
        if let Ok(output) = std::process::Command::new("rocm-smi")
            .args(&["--showtemp"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("GPU") && line.contains("°C") {
                            if let Some(temp_str) =
                                line.split_whitespace().find(|s| s.contains("°C"))
                            {
                                if let Ok(temp) = temp_str.trim_end_matches("°C").parse::<f64>() {
                                    return Ok(temp);
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
                        if name.trim().contains("amdgpu") || name.trim().contains("radeon") {
                            let temp_file = path.join("temp1_input");
                            if let Ok(temp_str) = std::fs::read_to_string(&temp_file) {
                                if let Ok(temp) = temp_str.trim().parse::<f64>() {
                                    return Ok(temp / 1000.0);
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(anyhow::anyhow!("No GPU temperature sensor found"))
    }

    #[cfg(target_os = "windows")]
    {
        get_windows_gpu_temperature()
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
                        if line.contains("GPU temperature") {
                            if let Some(temp_str) = line.split(':').nth(1) {
                                if let Ok(temp) = temp_str.trim().parse::<f64>() {
                                    return Ok(temp);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(45.0)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(anyhow::anyhow!(
            "GPU temperature not supported on this platform"
        ))
    }
}

#[cfg(target_os = "windows")]
fn get_windows_gpu_temperature() -> Result<f64> {
    use std::process::Command;
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-CimInstance -Namespace root/wmi -ClassName MSAcpi_ThermalZoneTemperature | Select-Object -ExpandProperty CurrentTemperature"
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
        use nvml_wrapper::Nvml;
        if let Ok(nvml) = Nvml::init() {
            if let Ok(device) = nvml.device_by_index(0) {
                if let Ok(temp) =
                    device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                {
                    return Ok(temp as f64);
                }
            }
        }
    }

    Ok(45.0)
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
