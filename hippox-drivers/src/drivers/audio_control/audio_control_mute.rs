//! Audio mute skill

use super::common::mute;
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
pub struct AudioControlMuteDriver;

#[async_trait::async_trait]
impl Driver for AudioControlMuteDriver {
    fn name(&self) -> &str {
        "audio_control_mute"
    }

    fn description(&self) -> &str {
        "Mute system audio"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to mute all system sounds."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "audio_control_mute"
        })
    }

    fn example_output(&self) -> String {
        "Audio muted".to_string()
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
        mute()?;
        Ok("Audio muted".to_string())
    }
}
