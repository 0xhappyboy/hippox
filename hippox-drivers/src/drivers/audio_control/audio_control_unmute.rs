//! Audio unmute driver
//!
//! This driver provides functionality to unmute all system audio.
use super::common::unmute;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for unmuting system audio
#[derive(Debug)]
pub struct AudioControlUnmuteDriver;
#[async_trait::async_trait]
impl Driver for AudioControlUnmuteDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "audio_control_unmute"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Unmute system audio"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to unmute system sounds."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "audio_control_unmute"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Audio unmuted".to_string()
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
        debug!("Executing audio_control_unmute driver");
        unmute().map_err(|e| DriverError::execution(format!("Failed to unmute audio: {}", e)))?;
        info!("Audio unmuted");
        Ok("Audio unmuted".to_string())
    }
}
