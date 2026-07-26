//! Audio input volume set driver
//!
//! This driver provides functionality to set the microphone input
//! volume level on the system.
use super::common::set_input_volume;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for setting microphone input volume
#[derive(Debug)]
pub struct AudioControlInputVolumeSetDriver;
#[async_trait::async_trait]
impl Driver for AudioControlInputVolumeSetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "audio_control_input_volume_set"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set the microphone input volume level"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to adjust microphone sensitivity (0-100)."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "volume".to_string(),
            param_type: "integer".to_string(),
            description: "Microphone volume level from 0 to 100".to_string(),
            required: true,
            default: None,
            example: Some(Value::Number(75.into())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "audio_control_input_volume_set",
            "parameters": {
                "volume": 75
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Input volume set to 75%".to_string()
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
        debug!("Executing audio_control_input_volume_set driver");
        let volume = parameters.get("volume").and_then(|v| v.as_u64()).ok_or_else(|| {
            debug!("Missing 'volume' parameter");
            DriverError::missing_parameter("volume")
        })? as u32;
        let volume = volume.clamp(0, 100);
        debug!("Setting input volume to {}%", volume);
        set_input_volume(volume).map_err(|e| DriverError::execution(format!("Failed to set input volume: {}", e)))?;
        info!("Input volume set to {}%", volume);
        Ok(format!("Input volume set to {}%", volume))
    }
}
