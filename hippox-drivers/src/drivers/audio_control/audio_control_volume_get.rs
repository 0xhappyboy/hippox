//! Audio volume get skill

use super::common::get_volume;
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
pub struct AudioControlVolumeGetDriver;

#[async_trait::async_trait]
impl Driver for AudioControlVolumeGetDriver {
    fn name(&self) -> &str {
        "audio_control_volume_get"
    }

    fn description(&self) -> &str {
        "Get the current system volume level"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to query the current audio volume (0-100)."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "audio_control_volume_get"
        })
    }

    fn example_output(&self) -> String {
        "Current volume: 65%".to_string()
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
        let volume = get_volume()?;
        Ok(format!("Current volume: {}%", volume))
    }
}
