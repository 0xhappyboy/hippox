//! Audio input device list driver
//!
//! This driver provides functionality to list all available audio input
//! devices (microphones) on the system.
use super::common::list_input_devices;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing audio input devices
#[derive(Debug)]
pub struct AudioControlInputDeviceListDriver;
#[async_trait::async_trait]
impl Driver for AudioControlInputDeviceListDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "audio_control_input_device_list"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "List all available audio input devices (microphones)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to see what microphone devices are available."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "audio_control_input_device_list"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Found 2 input devices:\n1. Default Microphone (default)\n2. Microphone Array".to_string()
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
        debug!("Executing audio_control_input_device_list driver");
        let devices = list_input_devices().map_err(|e| DriverError::execution(format!("Failed to list input devices: {}", e)))?;
        if devices.is_empty() {
            info!("No input devices found");
            return Ok("No input devices found".to_string());
        }
        let mut result = format!("Found {} input devices:\n", devices.len());
        for (i, device) in devices.iter().enumerate() {
            let default_marker = if device.is_default { " (default)" } else { "" };
            result.push_str(&format!("{}. {}{} (ID: {})\n", i + 1, device.name, default_marker, device.id));
        }
        info!("Listed {} input devices", devices.len());
        Ok(result)
    }
}
