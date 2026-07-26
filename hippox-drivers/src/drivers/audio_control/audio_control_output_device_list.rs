//! Audio output device list driver
//!
//! This driver provides functionality to list all available audio
//! output devices (speakers, headphones, etc.) on the system.
use super::common::list_output_devices;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing audio output devices
#[derive(Debug)]
pub struct AudioControlOutputDeviceListDriver;
#[async_trait::async_trait]
impl Driver for AudioControlOutputDeviceListDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "audio_control_output_device_list"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "List all available audio output devices"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to see what audio output devices are available (speakers, headphones, etc.)."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "audio_control_output_device_list"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Found 3 output devices:\n1. Default Output Device (default)\n2. Speakers\n3. Headphones".to_string()
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        DriverCategory::Audio
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing audio_control_output_device_list driver");
        let devices = list_output_devices().map_err(|e| DriverError::execution(format!("Failed to list output devices: {}", e)))?;
        if devices.is_empty() {
            info!("No output devices found");
            return Ok("No output devices found".to_string());
        }
        let mut result = format!("Found {} output devices:\n", devices.len());
        for (i, device) in devices.iter().enumerate() {
            let default_marker = if device.is_default { " (default)" } else { "" };
            result.push_str(&format!("{}. {}{} (ID: {})\n", i + 1, device.name, default_marker, device.id));
        }
        info!("Listed {} output devices", devices.len());
        Ok(result)
    }
}
