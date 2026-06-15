// audio_control/audio_control_unmute.rs
//! Audio unmute skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::types::{Skill, SkillParameter};
use super::common::unmute;

#[derive(Debug)]
pub struct AudioControlUnmuteSkill;

#[async_trait::async_trait]
impl Skill for AudioControlUnmuteSkill {
    fn name(&self) -> &str {
        "audio_control_unmute"
    }

    fn description(&self) -> &str {
        "Unmute system audio"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to unmute system sounds."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
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

    fn category(&self) -> &str {
        "audio_control"
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        unmute()?;
        Ok("Audio unmuted".to_string())
    }
}