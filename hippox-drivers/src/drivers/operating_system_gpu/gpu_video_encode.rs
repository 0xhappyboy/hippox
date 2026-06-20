//! GPU video encode engine usage driver

use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for getting GPU video encode engine usage
#[derive(Debug)]
pub struct GpuVideoEncodeDriver;

#[async_trait::async_trait]
impl Driver for GpuVideoEncodeDriver {
    fn name(&self) -> &str {
        "gpu_video_encode"
    }

    fn description(&self) -> &str {
        "Get GPU video encode engine utilization percentage"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to monitor video encoding performance"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "gpu_video_encode",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        "GPU Video Encode: 45.0%".to_string()
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
        let usage = get_video_encode_usage()?;
        Ok(format!("GPU Video Encode: {:.1}%", usage))
    }
}

fn get_video_encode_usage() -> Result<f32> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(&["--query-gpu", "utilization.encoder"])
            .args(&["--format", "csv,noheader"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(line) = output_str.lines().next() {
                        if let Ok(usage) = line.trim().trim_end_matches('%').parse::<f32>() {
                            return Ok(usage);
                        }
                    }
                }
            }
        }
        if let Ok(output) = std::process::Command::new("rocm-smi")
            .args(&["--showencoder"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("Encoder") && line.contains("%") {
                            if let Some(usage) = line
                                .split_whitespace()
                                .find(|s| s.contains("%"))
                                .and_then(|s| s.trim_end_matches('%').parse::<f32>().ok())
                            {
                                return Ok(usage);
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
                            let encode_path = path.join("device/encoder_busy_percent");
                            if let Ok(load_str) = std::fs::read_to_string(&encode_path) {
                                if let Ok(usage) = load_str.trim().parse::<f32>() {
                                    return Ok(usage);
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
        Ok(0.0)
    }

    #[cfg(target_os = "macos")]
    {
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
    fn test_gpu_video_encode_metadata() {
        let driver = GpuVideoEncodeDriver;
        assert_eq!(driver.name(), "gpu_video_encode");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemGpu);
    }
}
