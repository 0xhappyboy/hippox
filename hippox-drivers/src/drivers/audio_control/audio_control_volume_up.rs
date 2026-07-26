//! Audio volume up driver
//!
//! This driver provides functionality to increase the system volume
//! by a specified amount.
use super::common::volume_up;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for increasing system volume
#[derive(Debug)]
pub struct AudioControlVolumeUpDriver;
#[async_trait::async_trait]
impl Driver for AudioControlVolumeUpDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "audio_control_volume_up"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Increase system volume by a specified amount"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to increase the volume. Default delta is 10 if not specified."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "delta".to_string(),
            param_type: "integer".to_string(),
            description: "Amount to increase by (0-100)".to_string(),
            required: false,
            default: Some(Value::Number(10.into())),
            example: Some(Value::Number(20.into())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "audio_control_volume_up",
            "parameters": {
                "delta": 10
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Volume increased by 10%".to_string()
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
        debug!("Executing audio_control_volume_up driver");
        let delta = parameters.get("delta").and_then(|v| v.as_u64()).unwrap_or(10) as u32;
        debug!("Increasing volume by {}%", delta);
        volume_up(delta).map_err(|e| DriverError::execution(format!("Failed to increase volume: {}", e)))?;
        info!("Volume increased by {}%", delta);
        Ok(format!("Volume increased by {}%", delta))
    }
}
