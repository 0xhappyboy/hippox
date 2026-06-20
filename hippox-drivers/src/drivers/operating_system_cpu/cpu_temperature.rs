//! CPU temperature driver

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for getting CPU temperature
#[derive(Debug)]
pub struct CpuTemperatureDriver;

#[async_trait::async_trait]
impl Driver for CpuTemperatureDriver {
    fn name(&self) -> &str {
        "cpu_temperature"
    }

    fn description(&self) -> &str {
        "Get CPU temperature (requires sensor support)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to monitor CPU temperature for thermal management"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "cpu_temperature",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        r#"CPU Temperature: 45.0°C"#.to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemCpu
    }

    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let temp = get_cpu_temperature()?;
        Ok(format!("CPU Temperature: {:.1}°C", temp))
    }
}

fn get_cpu_temperature() -> Result<f64> {
    #[cfg(target_os = "linux")]
    {
        // Try thermal zones
        let thermal_path = "/sys/class/thermal/thermal_zone0/temp";
        if let Ok(temp_str) = std::fs::read_to_string(thermal_path) {
            if let Ok(temp) = temp_str.trim().parse::<f64>() {
                return Ok(temp / 1000.0);
            }
        }

        // Try hwmon
        let hwmon_path = "/sys/class/hwmon";
        if let Ok(entries) = std::fs::read_dir(hwmon_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let temp_file = path.join("temp1_input");
                    if let Ok(temp_str) = std::fs::read_to_string(&temp_file) {
                        if let Ok(temp) = temp_str.trim().parse::<f64>() {
                            return Ok(temp / 1000.0);
                        }
                    }
                }
            }
        }

        Err(anyhow::anyhow!("No temperature sensor found"))
    }

    #[cfg(target_os = "windows")]
    {
        get_windows_temp_via_powershell()
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        // Use powermetrics or istats if available
        let output = Command::new("sysctl")
            .args(&["-n", "hw.sensors.cpu0.temperature"])
            .output();

        if let Ok(output) = output {
            if let Ok(temp_str) = String::from_utf8(output.stdout) {
                if let Ok(temp) = temp_str.trim().parse::<f64>() {
                    return Ok(temp);
                }
            }
        }

        // Fallback: try using powermetrics
        let output = Command::new("sudo")
            .args(&["powermetrics", "-n", "1"])
            .output();

        if let Ok(output) = output {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("CPU temperature") {
                        if let Some(temp_str) = line.split(':').nth(1) {
                            if let Ok(temp) = temp_str.trim().parse::<f64>() {
                                return Ok(temp);
                            }
                        }
                    }
                }
            }
        }

        Err(anyhow::anyhow!("No temperature sensor found"))
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(anyhow::anyhow!(
            "CPU temperature not supported on this platform"
        ))
    }
}

#[cfg(target_os = "windows")]
fn get_windows_temp_via_powershell() -> Result<f64> {
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
    Err(anyhow::anyhow!(
        "No temperature sensor found (PowerShell query failed)"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_temperature_metadata() {
        let driver = CpuTemperatureDriver;
        assert_eq!(driver.name(), "cpu_temperature");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemCpu);
    }
}
