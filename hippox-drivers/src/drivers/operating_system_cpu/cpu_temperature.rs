//! CPU temperature driver module
//!
//! This module provides functionality to get CPU temperature readings
//! from thermal sensors (requires sensor support).
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
/// Driver for getting CPU temperature
#[derive(Debug)]
pub struct CpuTemperatureDriver;
#[async_trait::async_trait]
impl Driver for CpuTemperatureDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "cpu_temperature";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get CPU temperature (requires sensor support)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to monitor CPU temperature for thermal management";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "cpu_temperature",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"CPU Temperature: 45.0°C"#.to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemCpu;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing cpu_temperature driver");
        let temp = get_cpu_temperature()?;
        let result = format!("CPU Temperature: {:.1}°C", temp);
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
fn get_cpu_temperature() -> DriverResult<f64> {
    #[cfg(target_os = "linux")]
    {
        let thermal_path = "/sys/class/thermal/thermal_zone0/temp";
        if let Ok(temp_str) = std::fs::read_to_string(thermal_path) {
            if let Ok(temp) = temp_str.trim().parse::<f64>() {
                debug!("Read temperature from thermal_zone0: {}°C", temp / 1000.0);
                return Ok(temp / 1000.0);
            }
        }
        let hwmon_path = "/sys/class/hwmon";
        if let Ok(entries) = std::fs::read_dir(hwmon_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let temp_file = path.join("temp1_input");
                    if let Ok(temp_str) = std::fs::read_to_string(&temp_file) {
                        if let Ok(temp) = temp_str.trim().parse::<f64>() {
                            debug!("Read temperature from hwmon: {}°C", temp / 1000.0);
                            return Ok(temp / 1000.0);
                        }
                    }
                }
            }
        }
        return Err(DriverError::execution("No temperature sensor found".to_string()));
    }
    #[cfg(target_os = "windows")]
    {
        return get_windows_temp_via_powershell();
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("sysctl").args(&["-n", "hw.sensors.cpu0.temperature"]).output();
        if let Ok(output) = output {
            if let Ok(temp_str) = String::from_utf8(output.stdout) {
                if let Ok(temp) = temp_str.trim().parse::<f64>() {
                    debug!("Read temperature from sysctl: {}°C", temp);
                    return Ok(temp);
                }
            }
        }
        let output = Command::new("sudo").args(&["powermetrics", "-n", "1"]).output();
        if let Ok(output) = output {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("CPU temperature") {
                        if let Some(temp_str) = line.split(':').nth(1) {
                            if let Ok(temp) = temp_str.trim().parse::<f64>() {
                                debug!("Read temperature from powermetrics: {}°C", temp);
                                return Ok(temp);
                            }
                        }
                    }
                }
            }
        }
        return Err(DriverError::execution("No temperature sensor found".to_string()));
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        return Err(DriverError::execution("CPU temperature not supported on this platform".to_string()));
    }
}
#[cfg(target_os = "windows")]
fn get_windows_temp_via_powershell() -> DriverResult<f64> {
    use std::process::Command;
    debug!("Getting CPU temperature via PowerShell on Windows");
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-CimInstance -Namespace root/wmi -ClassName MSAcpi_ThermalZoneTemperature | Select-Object -ExpandProperty CurrentTemperature",
        ])
        .output()
        .map_err(|e| DriverError::execution(format!("PowerShell query failed: {}", e)))?;
    if output.status.success() {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            for line in output_str.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if let Ok(temp_raw) = trimmed.parse::<f64>() {
                        let celsius = (temp_raw / 10.0) - 273.15;
                        if celsius > 0.0 && celsius < 100.0 {
                            debug!("Read temperature from WMI: {}°C", celsius);
                            return Ok(celsius);
                        }
                    }
                }
            }
        }
    }
    return Err(DriverError::execution("No temperature sensor found (PowerShell query failed)".to_string()));
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
