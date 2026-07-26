//! Audio mute driver
//!
//! This driver provides functionality to mute all system audio.
use super::common::mute;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for muting system audio
#[derive(Debug)]
pub struct AudioControlMuteDriver;
#[async_trait::async_trait]
impl Driver for AudioControlMuteDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "audio_control_mute"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Mute system audio"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to mute all system sounds."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "audio_control_mute"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Audio muted".to_string()
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
        debug!("Executing audio_control_mute driver");
        mute().map_err(|e| DriverError::execution(format!("Failed to mute audio: {}", e)))?;
        info!("Audio muted");
        Ok("Audio muted".to_string())
    }
}
