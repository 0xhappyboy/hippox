// audio_control/audio_control_input_volume_set.rs
//! Audio input volume set skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use super::common::set_input_volume;

#[derive(Debug)]
pub struct AudioControlInputVolumeSetSkill;

#[async_trait::async_trait]
impl Skill for AudioControlInputVolumeSetSkill {
    fn name(&self) -> &str {
        "audio_control_input_volume_set"
    }

    fn description(&self) -> &str {
        "Set the microphone input volume level"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to adjust microphone sensitivity (0-100)."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "volume".to_string(),
                param_type: "integer".to_string(),
                description: "Microphone volume level from 0 to 100".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(75.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "audio_control_input_volume_set",
            "parameters": {
                "volume": 75
            }
        })
    }

    fn example_output(&self) -> String {
        "Input volume set to 75%".to_string()
    }

    fn category(&self) -> &str {
        "audio_control"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let volume = parameters.get("volume")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'volume' parameter"))? as u32;
        
        let volume = volume.clamp(0, 100);
        set_input_volume(volume)?;
        
        Ok(format!("Input volume set to {}%", volume))
    }
}