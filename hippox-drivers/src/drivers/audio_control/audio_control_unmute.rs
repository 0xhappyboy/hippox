//! Audio unmute skill

use super::common::unmute;
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
pub struct AudioControlUnmuteDriver;

#[async_trait::async_trait]
impl Driver for AudioControlUnmuteDriver {
    fn name(&self) -> &str {
        "audio_control_unmute"
    }

    fn description(&self) -> &str {
        "Unmute system audio"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to unmute system sounds."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "audio_control_unmute"
        })
    }

    fn example_output(&self) -> String {
        "Audio unmuted".to_string()
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
        unmute()?;
        Ok("Audio unmuted".to_string())
    }
}
