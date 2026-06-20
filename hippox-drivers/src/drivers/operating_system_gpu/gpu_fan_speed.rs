//! GPU fan speed driver

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for getting GPU fan speed
#[derive(Debug)]
pub struct GpuFanSpeedDriver;

#[async_trait::async_trait]
impl Driver for GpuFanSpeedDriver {
    fn name(&self) -> &str {
        "gpu_fan_speed"
    }

    fn description(&self) -> &str {
        "Get GPU fan speed percentage"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to monitor GPU cooling performance"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "gpu_fan_speed",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        "GPU Fan Speed: 45.0%".to_string()
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
        let speed = get_gpu_fan_speed()?;
        Ok(format!("GPU Fan Speed: {:.1}%", speed))
    }
}

fn get_gpu_fan_speed() -> Result<f32> {
    #[cfg(target_os = "linux")]
    {
        // Try NVIDIA
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(&["--query-gpu", "fan.speed"])
            .args(&["--format", "csv,noheader"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(line) = output_str.lines().next() {
                        if let Ok(speed) = line.trim().trim_end_matches('%').parse::<f32>() {
                            return Ok(speed);
                        }
                    }
                }
            }
        }

        // Try AMD via hwmon
        let hwmon_path = "/sys/class/hwmon";
        if let Ok(entries) = std::fs::read_dir(hwmon_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Ok(name) = std::fs::read_to_string(path.join("name")) {
                        if name.trim().contains("amdgpu") || name.trim().contains("radeon") {
                            let fan_file = path.join("pwm1");
                            if let Ok(fan_str) = std::fs::read_to_string(&fan_file) {
                                if let Ok(fan) = fan_str.trim().parse::<f32>() {
                                    // pwm values are typically 0-255
                                    return Ok((fan / 255.0) * 100.0);
                                }
                            }
                            let fan_file2 = path.join("fan1_input");
                            if let Ok(fan_str) = std::fs::read_to_string(&fan_file2) {
                                if let Ok(fan) = fan_str.trim().parse::<f32>() {
                                    // Some AMD GPUs report RPM directly
                                    return Ok(fan);
                                }
                            }
                            let fan_file3 = path.join("pwm2");
                            if let Ok(fan_str) = std::fs::read_to_string(&fan_file3) {
                                if let Ok(fan) = fan_str.trim().parse::<f32>() {
                                    return Ok((fan / 255.0) * 100.0);
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
        get_windows_gpu_fan_speed()
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: Try SMC via istats
        if let Ok(output) = std::process::Command::new("istats")
            .args(&["fan", "speed"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("Fan") && line.contains("RPM") {
                            if let Some(speed_str) = line.split(':').nth(1) {
                                if let Ok(rpm) = speed_str.trim().parse::<f32>() {
                                    // Assume max 5000 RPM
                                    return Ok((rpm / 5000.0) * 100.0);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Try via smc command
        if let Ok(output) = std::process::Command::new("smc").args(&["-f"]).output() {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("Fan") && line.contains("RPM") {
                            if let Some(rpm_str) = line.split_whitespace().last() {
                                if let Ok(rpm) = rpm_str.parse::<f32>() {
                                    return Ok((rpm / 5000.0) * 100.0);
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
fn get_windows_gpu_fan_speed() -> Result<f32> {
    use std::process::Command;

    // Try NVIDIA via nvidia-smi
    if let Ok(output) = Command::new("nvidia-smi")
        .args(&["--query-gpu", "fan.speed"])
        .args(&["--format", "csv,noheader"])
        .output()
    {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                if let Some(line) = output_str.lines().next() {
                    if let Ok(speed) = line.trim().trim_end_matches('%').parse::<f32>() {
                        return Ok(speed);
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
                if let Ok(speed) = device.fan_speed(0) {
                    return Ok(speed as f32);
                }
            }
        }
    }

    // Try WMI via PowerShell
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-CimInstance -Namespace root/wmi -ClassName MSAcpi_ThermalZoneTemperature | Select-Object -ExpandProperty CurrentTemperature"
        ])
        .output();

    // Also try OpenHardwareMonitor WMI
    let output_ohm = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-WmiObject -Namespace root/OpenHardwareMonitor -Class Sensor | Where-Object {$_.SensorType -eq 'Fan'} | Select-Object -ExpandProperty Value"
        ])
        .output();

    // Try LibreHardwareMonitor WMI
    let output_lhm = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-WmiObject -Namespace root/LibreHardwareMonitor -Class Sensor | Where-Object {$_.SensorType -eq 'Fan'} | Select-Object -ExpandProperty Value"
        ])
        .output();

    // Parse OpenHardwareMonitor results
    if let Ok(output) = output_ohm {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        if let Ok(speed) = trimmed.parse::<f32>() {
                            if speed > 0.0 && speed <= 100.0 {
                                return Ok(speed);
                            }
                        }
                    }
                }
            }
        }
    }

    // Parse LibreHardwareMonitor results
    if let Ok(output) = output_lhm {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        if let Ok(speed) = trimmed.parse::<f32>() {
                            if speed > 0.0 && speed <= 100.0 {
                                return Ok(speed);
                            }
                        }
                    }
                }
            }
        }
    }

    // Try via HWiNFO WMI if available
    let output_hwinfo = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-WmiObject -Namespace root/HWiNFO -Class Sensor | Where-Object {$_.SensorType -eq 'Fan'} | Select-Object -ExpandProperty Value"
        ])
        .output();

    if let Ok(output) = output_hwinfo {
        if output.status.success() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        if let Ok(speed) = trimmed.parse::<f32>() {
                            if speed > 0.0 && speed <= 100.0 {
                                return Ok(speed);
                            }
                        }
                    }
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
    fn test_gpu_fan_speed_metadata() {
        let driver = GpuFanSpeedDriver;
        assert_eq!(driver.name(), "gpu_fan_speed");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemGpu);
    }
}
