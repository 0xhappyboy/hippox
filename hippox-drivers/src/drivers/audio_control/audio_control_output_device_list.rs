//! Audio output device list skill

use super::common::list_output_devices;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct AudioControlOutputDeviceListDriver;

#[async_trait::async_trait]
impl Driver for AudioControlOutputDeviceListDriver {
    fn name(&self) -> &str {
        "audio_control_output_device_list"
    }

    fn description(&self) -> &str {
        "List all available audio output devices"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see what audio output devices are available (speakers, headphones, etc.)."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "audio_control_output_device_list"
        })
    }

    fn example_output(&self) -> String {
        "Found 3 output devices:\n1. Default Output Device (default)\n2. Speakers\n3. Headphones"
            .to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Audio
    }

    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let devices = list_output_devices()?;
        if devices.is_empty() {
            return Ok("No output devices found".to_string());
        }
        let mut result = format!("Found {} output devices:\n", devices.len());
        for (i, device) in devices.iter().enumerate() {
            let default_marker = if device.is_default { " (default)" } else { "" };
            result.push_str(&format!(
                "{}. {}{} (ID: {})\n",
                i + 1,
                device.name,
                default_marker,
                device.id
            ));
        }
        Ok(result)
    }
}
