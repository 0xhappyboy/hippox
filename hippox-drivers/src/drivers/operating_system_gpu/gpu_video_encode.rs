//! GPU video encode engine usage driver
//!
//! This driver provides functionality to get GPU video encode engine utilization percentage.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting GPU video encode engine usage
#[derive(Debug)]
pub struct GpuVideoEncodeDriver;
#[async_trait::async_trait]
impl Driver for GpuVideoEncodeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "gpu_video_encode"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get GPU video encode engine utilization percentage"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to monitor video encoding performance"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "gpu_video_encode",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "GPU Video Encode: 45.0%".to_string();
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
        debug!("Executing gpu_video_encode driver");
        let usage = get_video_encode_usage()?;
        info!("GPU video encode usage: {:.1}%", usage);
        return Ok(format!("GPU Video Encode: {:.1}%", usage));
    }
}
/// Gets video encode engine usage from the system
fn get_video_encode_usage() -> DriverResult<f32> {
    #[cfg(target_os = "linux")]
    {
        debug!("Getting GPU video encode usage on Linux");
        // Try NVIDIA
        debug!("Trying NVIDIA nvidia-smi for encode usage");
        if let Ok(output) =
            std::process::Command::new("nvidia-smi").args(&["--query-gpu", "utilization.encoder"]).args(&["--format", "csv,noheader"]).output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(line) = output_str.lines().next() {
                        if let Ok(usage) = line.trim().trim_end_matches('%').parse::<f32>() {
                            info!("NVIDIA encode usage: {:.1}%", usage);
                            return Ok(usage);
                        }
                    }
                }
            }
        }
        // Try AMD via rocm-smi
        debug!("Trying AMD rocm-smi for encode usage");
        if let Ok(output) = std::process::Command::new("rocm-smi").args(&["--showencoder"]).output() {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("Encoder") && line.contains("%") {
                            if let Some(usage) =
                                line.split_whitespace().find(|s| s.contains("%")).and_then(|s| s.trim_end_matches('%').parse::<f32>().ok())
                            {
                                info!("AMD rocm-smi encode usage: {:.1}%", usage);
                                return Ok(usage);
                            }
                        }
                    }
                }
            }
        }
        // Try AMD via hwmon
        debug!("Trying AMD hwmon for encode usage");
        let hwmon_path = "/sys/class/hwmon";
        if let Ok(entries) = std::fs::read_dir(hwmon_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Ok(name) = std::fs::read_to_string(path.join("name")) {
                        if name.trim().contains("amdgpu") {
                            let encode_path = path.join("device/encoder_busy_percent");
                            if let Ok(load_str) = std::fs::read_to_string(&encode_path) {
                                if let Ok(usage) = load_str.trim().parse::<f32>() {
                                    info!("AMD hwmon encode usage: {:.1}%", usage);
                                    return Ok(usage);
                                }
                            }
                        }
                    }
                }
            }
        }
        info!("GPU video encode usage not found on Linux");
        return Ok(0.0);
    }
    #[cfg(target_os = "windows")]
    {
        debug!("GPU video encode not available on Windows");
        return Ok(0.0);
    }
    #[cfg(target_os = "macos")]
    {
        debug!("GPU video encode not available on macOS");
        return Ok(0.0);
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        debug!("GPU video encode not supported on this platform");
        return Ok(0.0);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gpu_video_encode_metadata() {
        let driver = GpuVideoEncodeDriver;
        assert_eq!(driver.name(), "gpu_video_encode");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemGpu);
    }
}
