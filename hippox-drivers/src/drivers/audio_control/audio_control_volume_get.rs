//! Audio volume get driver
//!
//! This driver provides functionality to query the current system
//! volume level.
use super::common::get_volume;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting current system volume
#[derive(Debug)]
pub struct AudioControlVolumeGetDriver;
#[async_trait::async_trait]
impl Driver for AudioControlVolumeGetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "audio_control_volume_get"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the current system volume level"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to query the current audio volume (0-100)."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "audio_control_volume_get"
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Current volume: 65%".to_string()
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
        debug!("Executing audio_control_volume_get driver");
        let volume = get_volume().map_err(|e| DriverError::execution(format!("Failed to get volume: {}", e)))?;
        info!("Current volume: {}%", volume);
        Ok(format!("Current volume: {}%", volume))
    }
}
