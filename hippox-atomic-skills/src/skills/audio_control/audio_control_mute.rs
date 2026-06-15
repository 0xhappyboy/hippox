// audio_control/audio_control_mute.rs
//! Audio mute skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use super::common::mute;

#[derive(Debug)]
pub struct AudioControlMuteSkill;

#[async_trait::async_trait]
impl Skill for AudioControlMuteSkill {
    fn name(&self) -> &str {
        "audio_control_mute"
    }

    fn description(&self) -> &str {
        "Mute system audio"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to mute all system sounds."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
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

    fn category(&self) -> &str {
        "audio_control"
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        mute()?;
        Ok("Audio muted".to_string())
    }
}