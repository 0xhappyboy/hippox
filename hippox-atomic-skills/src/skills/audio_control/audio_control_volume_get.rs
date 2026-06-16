// audio_control/audio_control_volume_get.rs
//! Audio volume get skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_volume;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct AudioControlVolumeGetSkill;

#[async_trait::async_trait]
impl Skill for AudioControlVolumeGetSkill {
    fn name(&self) -> &str {
        "audio_control_volume_get"
    }

    fn description(&self) -> &str {
        "Get the current system volume level"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to query the current audio volume (0-100)."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
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

    fn category(&self) -> SkillCategory {
        SkillCategory::Audio
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let volume = get_volume()?;
        Ok(format!("Current volume: {}%", volume))
    }
}
