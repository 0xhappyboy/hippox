//! CPU frequency driver module
//!
//! This module provides functionality to get and set CPU frequency,
//! monitoring frequency scaling and performance modes.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use sysinfo::System;
use tracing::{debug, info};
/// Driver for getting/setting CPU frequency
#[derive(Debug)]
pub struct CpuFrequencyDriver;
#[async_trait::async_trait]
impl Driver for CpuFrequencyDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "cpu_frequency";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get current CPU frequency (and set if supported)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to monitor CPU frequency scaling or set performance mode";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "set_mhz".to_string(),
            param_type: "integer".to_string(),
            description: "Target frequency in MHz to set (if supported)".to_string(),
            required: false,
            default: None,
            example: Some(Value::Number(2600.into())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "cpu_frequency",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"Current CPU Frequency: 2.6 GHz
Min Frequency: 0.8 GHz
Max Frequency: 4.2 GHz"#
            .to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemCpu;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing cpu_frequency driver");
        let system = System::new_all();
        let cpus = system.cpus();
        if cpus.is_empty() {
            return Err(DriverError::execution("No CPU information available".to_string()));
        }
        let target_freq = parameters.get("set_mhz").and_then(|v| v.as_u64());
        if let Some(freq_mhz) = target_freq {
            debug!("Setting CPU frequency to {} MHz", freq_mhz);
            set_frequency(freq_mhz)?;
            let result = format!("CPU frequency set to {} MHz", freq_mhz);
            info!("{}", result);
            return Ok(result);
        } else {
            let current = cpus[0].frequency();
            let max_freq = cpus.iter().map(|c| c.frequency()).max().unwrap_or(current);
            let min_freq = cpus.iter().map(|c| c.frequency()).min().unwrap_or(current);
            let result = format!(
                "Current CPU Frequency: {:.1} GHz\n\
                 Min Frequency: {:.1} GHz\n\
                 Max Frequency: {:.1} GHz",
                current as f64 / 1000.0,
                min_freq as f64 / 1000.0,
                max_freq as f64 / 1000.0
            );
            info!("CPU frequency retrieved");
            return Ok(result);
        }
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
#[cfg(target_os = "linux")]
fn set_frequency(freq_mhz: u64) -> DriverResult<()> {
    let freq_khz = freq_mhz * 1000;
    let base_path = "/sys/devices/system/cpu/cpu0/cpufreq";
    if !Path::new(base_path).exists() {
        return Err(DriverError::execution("cpufreq not available on this system".to_string()));
    }
    let scaling_available_path = format!("{}/scaling_available_frequencies", base_path);
    if let Ok(content) = fs::read_to_string(&scaling_available_path) {
        let frequencies: Vec<u64> = content.split_whitespace().filter_map(|s| s.parse::<u64>().ok()).collect();
        if !frequencies.is_empty() && !frequencies.contains(&freq_khz) {
            return Err(DriverError::execution(format!(
                "Frequency {} MHz not available. Available: {:?}",
                freq_mhz,
                frequencies.iter().map(|f| f / 1000).collect::<Vec<_>>()
            )));
        }
    }
    let scaling_setspeed_path = format!("{}/scaling_setspeed", base_path);
    if Path::new(&scaling_setspeed_path).exists() {
        fs::write(&scaling_setspeed_path, freq_khz.to_string()).map_err(|e| DriverError::execution(format!("Failed to set frequency: {}", e)))?;
        return Ok(());
    }
    let scaling_governor_path = format!("{}/scaling_governor", base_path);
    if Path::new(&scaling_governor_path).exists() {
        fs::write(&scaling_governor_path, "userspace").map_err(|e| DriverError::execution(format!("Failed to set governor: {}", e)))?;
        fs::write(&scaling_setspeed_path, freq_khz.to_string()).map_err(|e| DriverError::execution(format!("Failed to set frequency: {}", e)))?;
        return Ok(());
    }
    let output = std::process::Command::new("cpufreq-set").args(&["-f", &freq_khz.to_string()]).output();
    if let Ok(output) = output {
        if output.status.success() {
            return Ok(());
        }
    }
    let output = std::process::Command::new("sudo").args(&["cpufreq-set", "-f", &freq_khz.to_string()]).output();
    if let Ok(output) = output {
        if output.status.success() {
            return Ok(());
        }
    }
    return Err(DriverError::execution("Failed to set CPU frequency. Requires root privileges and cpufreq support.".to_string()));
}
#[cfg(target_os = "windows")]
fn set_frequency(freq_mhz: u64) -> DriverResult<()> {
    use std::process::Command;
    let output = Command::new("powercfg")
        .args(&["/getactivescheme"])
        .output()
        .map_err(|e| DriverError::execution(format!("Failed to get power scheme: {}", e)))?;
    if output.status.success() {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            for line in output_str.lines() {
                if let Some(guid) = line.split_whitespace().find(|s| s.contains('{') && s.contains('}')) {
                    let _ = Command::new("powercfg")
                        .args(&[
                            "/setacvalueindex",
                            guid,
                            "54533251-82be-4824-96c1-47b60b740d00",
                            "bc5038f7-23e0-4960-96da-33abaf5935ec",
                            &freq_mhz.to_string(),
                        ])
                        .output();
                    let _ = Command::new("powercfg")
                        .args(&[
                            "/setdcvalueindex",
                            guid,
                            "54533251-82be-4824-96c1-47b60b740d00",
                            "bc5038f7-23e0-4960-96da-33abaf5935ec",
                            &freq_mhz.to_string(),
                        ])
                        .output();
                    let _ = Command::new("powercfg").args(&["/setactive", guid]).output();
                    return Ok(());
                }
            }
        }
    }
    return Err(DriverError::execution("Setting CPU frequency on Windows requires admin privileges and specific hardware support.".to_string()));
}
#[cfg(target_os = "macos")]
fn set_frequency(freq_mhz: u64) -> DriverResult<()> {
    use std::process::Command;
    let output = Command::new("sudo").args(&["pmset", "-a", "cpu_clock", &freq_mhz.to_string()]).output();
    if let Ok(output) = output {
        if output.status.success() {
            return Ok(());
        }
    }
    return Err(DriverError::execution("Setting CPU frequency on macOS requires root privileges.".to_string()));
}
#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn set_frequency(_freq_mhz: u64) -> DriverResult<()> {
    return Err(DriverError::execution("Setting CPU frequency is not supported on this platform".to_string()));
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cpu_frequency_metadata() {
        let driver = CpuFrequencyDriver;
        assert_eq!(driver.name(), "cpu_frequency");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemCpu);
    }
}
