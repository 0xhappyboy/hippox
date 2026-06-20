//! CPU frequency driver

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::System;

/// Driver for getting/setting CPU frequency
#[derive(Debug)]
pub struct CpuFrequencyDriver;

#[async_trait::async_trait]
impl Driver for CpuFrequencyDriver {
    fn name(&self) -> &str {
        "cpu_frequency"
    }

    fn description(&self) -> &str {
        "Get current CPU frequency (and set if supported)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to monitor CPU frequency scaling or set performance mode"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "set_mhz".to_string(),
            param_type: "integer".to_string(),
            description: "Target frequency in MHz to set (if supported)".to_string(),
            required: false,
            default: None,
            example: Some(Value::Number(2600.into())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "cpu_frequency",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        r#"Current CPU Frequency: 2.6 GHz
Min Frequency: 0.8 GHz
Max Frequency: 4.2 GHz"#
            .to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemCpu
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let system = System::new_all();
        let cpus = system.cpus();

        if cpus.is_empty() {
            return Err(anyhow::anyhow!("No CPU information available"));
        }

        let target_freq = parameters.get("set_mhz").and_then(|v| v.as_u64());

        if let Some(freq_mhz) = target_freq {
            set_frequency(freq_mhz)?;
            Ok(format!("CPU frequency set to {} MHz", freq_mhz))
        } else {
            let current = cpus[0].frequency();
            let max_freq = cpus.iter().map(|c| c.frequency()).max().unwrap_or(current);
            let min_freq = cpus.iter().map(|c| c.frequency()).min().unwrap_or(current);

            let output = format!(
                "Current CPU Frequency: {:.1} GHz\n\
                 Min Frequency: {:.1} GHz\n\
                 Max Frequency: {:.1} GHz",
                current as f64 / 1000.0,
                min_freq as f64 / 1000.0,
                max_freq as f64 / 1000.0
            );
            Ok(output)
        }
    }
}

#[cfg(target_os = "linux")]
fn set_frequency(freq_mhz: u64) -> Result<()> {
    use std::fs;
    use std::path::Path;

    let freq_khz = freq_mhz * 1000;

    // Try to find cpufreq directory
    let base_path = "/sys/devices/system/cpu/cpu0/cpufreq";
    if !Path::new(base_path).exists() {
        return Err(anyhow::anyhow!("cpufreq not available on this system"));
    }

    // Check if scaling is available
    let scaling_available_path = format!("{}/scaling_available_frequencies", base_path);
    if let Ok(content) = fs::read_to_string(&scaling_available_path) {
        let frequencies: Vec<u64> = content
            .split_whitespace()
            .filter_map(|s| s.parse::<u64>().ok())
            .collect();

        if !frequencies.is_empty() && !frequencies.contains(&freq_khz) {
            return Err(anyhow::anyhow!(
                "Frequency {} MHz not available. Available: {:?}",
                freq_mhz,
                frequencies.iter().map(|f| f / 1000).collect::<Vec<_>>()
            ));
        }
    }

    // Try to set frequency via scaling_setspeed
    let scaling_setspeed_path = format!("{}/scaling_setspeed", base_path);
    if Path::new(&scaling_setspeed_path).exists() {
        fs::write(&scaling_setspeed_path, freq_khz.to_string())?;
        return Ok(());
    }

    // Try via scaling_governor (userspace)
    let scaling_governor_path = format!("{}/scaling_governor", base_path);
    if Path::new(&scaling_governor_path).exists() {
        // First set governor to userspace
        fs::write(&scaling_governor_path, "userspace")?;
        // Then set frequency
        fs::write(&scaling_setspeed_path, freq_khz.to_string())?;
        return Ok(());
    }

    // Fallback to cpufreq-set command
    let output = std::process::Command::new("cpufreq-set")
        .args(&["-f", &freq_khz.to_string()])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            return Ok(());
        }
    }

    // Try with sudo
    let output = std::process::Command::new("sudo")
        .args(&["cpufreq-set", "-f", &freq_khz.to_string()])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            return Ok(());
        }
    }

    Err(anyhow::anyhow!(
        "Failed to set CPU frequency. Requires root privileges and cpufreq support."
    ))
}

#[cfg(target_os = "windows")]
fn set_frequency(freq_mhz: u64) -> Result<()> {
    // Windows: Use powercfg to set power scheme
    use std::process::Command;

    // Get current power scheme
    let output = Command::new("powercfg")
        .args(&["/getactivescheme"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                // Try to extract GUID
                for line in output_str.lines() {
                    if let Some(guid) = line
                        .split_whitespace()
                        .find(|s| s.contains('{') && s.contains('}'))
                    {
                        // Set processor performance max to target frequency
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
                        let _ = Command::new("powercfg")
                            .args(&["/setactive", guid])
                            .output();
                        return Ok(());
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!(
        "Setting CPU frequency on Windows requires admin privileges and specific hardware support."
    ))
}

#[cfg(target_os = "macos")]
fn set_frequency(freq_mhz: u64) -> Result<()> {
    use std::process::Command;

    // macOS: Use pmset
    let output = Command::new("sudo")
        .args(&["pmset", "-a", "cpu_clock", &freq_mhz.to_string()])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            return Ok(());
        }
    }

    Err(anyhow::anyhow!(
        "Setting CPU frequency on macOS requires root privileges."
    ))
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn set_frequency(_freq_mhz: u64) -> Result<()> {
    Err(anyhow::anyhow!(
        "Setting CPU frequency is not supported on this platform"
    ))
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
