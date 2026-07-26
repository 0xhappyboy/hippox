//! Audio output device set driver
//!
//! This driver provides functionality to set the active audio
//! output device on the system.
use super::common::set_output_device;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for setting the active audio output device
#[derive(Debug)]
pub struct AudioControlOutputDeviceSetDriver;
#[async_trait::async_trait]
impl Driver for AudioControlOutputDeviceSetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "audio_control_output_device_set"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set the active audio output device"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to switch between speakers, headphones, or other output devices."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "device_id".to_string(),
            param_type: "string".to_string(),
            description: "Device ID from output device list".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("headphones".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "audio_control_output_device_set",
            "parameters": {
                "device_id": "headphones"
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Output device set to headphones".to_string()
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        DriverCategory::Audio
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing audio_control_output_device_set driver");
        let device_id = parameters.get("device_id").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'device_id' parameter");
            DriverError::missing_parameter("device_id")
        })?;
        debug!("Setting output device to: {}", device_id);
        set_output_device(device_id).map_err(|e| DriverError::execution(format!("Failed to set output device: {}", e)))?;
        info!("Output device set to: {}", device_id);
        Ok(format!("Output device set to {}", device_id))
    }
}
