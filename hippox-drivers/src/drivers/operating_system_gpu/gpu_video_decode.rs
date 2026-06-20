//! GPU video decode engine usage driver

use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for getting GPU video decode engine usage
#[derive(Debug)]
pub struct GpuVideoDecodeDriver;

#[async_trait::async_trait]
impl Driver for GpuVideoDecodeDriver {
    fn name(&self) -> &str {
        "gpu_video_decode"
    }

    fn description(&self) -> &str {
        "Get GPU video decode engine utilization percentage"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to monitor video decoding performance"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "gpu_video_decode",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        "GPU Video Decode: 35.0%".to_string()
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
        let usage = get_video_decode_usage()?;
        Ok(format!("GPU Video Decode: {:.1}%", usage))
    }
}

fn get_video_decode_usage() -> Result<f32> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(&["--query-gpu", "utilization.decoder"])
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
            .args(&["--showdecoder"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("Decoder") && line.contains("%") {
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
                            let decode_path = path.join("device/decoder_busy_percent");
                            if let Ok(load_str) = std::fs::read_to_string(&decode_path) {
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
    fn test_gpu_video_decode_metadata() {
        let driver = GpuVideoDecodeDriver;
        assert_eq!(driver.name(), "gpu_video_decode");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemGpu);
    }
}
