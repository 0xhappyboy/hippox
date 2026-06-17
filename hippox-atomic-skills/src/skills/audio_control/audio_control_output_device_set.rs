// audio_control/audio_control_output_device_set.rs
//! Audio output device set skill

use super::common::set_output_device;
use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct AudioControlOutputDeviceSetSkill;

#[async_trait::async_trait]
impl Skill for AudioControlOutputDeviceSetSkill {
    fn name(&self) -> &str {
        "audio_control_output_device_set"
    }

    fn description(&self) -> &str {
        "Set the active audio output device"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to switch between speakers, headphones, or other output devices."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "device_id".to_string(),
            param_type: "string".to_string(),
            description: "Device ID from output device list".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("headphones".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "audio_control_output_device_set",
            "parameters": {
                "device_id": "headphones"
            }
        })
    }

    fn example_output(&self) -> String {
        "Output device set to headphones".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Audio
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let device_id = parameters
            .get("device_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'device_id' parameter"))?;
        set_output_device(device_id)?;
        Ok(format!("Output device set to {}", device_id))
    }
}
