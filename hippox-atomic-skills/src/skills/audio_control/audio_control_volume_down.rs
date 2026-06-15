// audio_control/audio_control_volume_down.rs
//! Audio volume down skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use super::common::volume_down;

#[derive(Debug)]
pub struct AudioControlVolumeDownSkill;

#[async_trait::async_trait]
impl Skill for AudioControlVolumeDownSkill {
    fn name(&self) -> &str {
        "audio_control_volume_down"
    }

    fn description(&self) -> &str {
        "Decrease system volume by a specified amount"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to decrease the volume. Default delta is 10 if not specified."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "delta".to_string(),
                param_type: "integer".to_string(),
                description: "Amount to decrease by (0-100)".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(20.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "audio_control_volume_down",
            "parameters": {
                "delta": 10
            }
        })
    }

    fn example_output(&self) -> String {
        "Volume decreased by 10%".to_string()
    }

    fn category(&self) -> &str {
        "audio_control"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let delta = parameters.get("delta")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as u32;
        
        volume_down(delta)?;
        
        Ok(format!("Volume decreased by {}%", delta))
    }
}