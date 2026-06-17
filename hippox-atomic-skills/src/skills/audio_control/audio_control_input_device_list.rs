// audio_control/audio_control_input_device_list.rs
//! Audio input device list skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::list_input_devices;
use crate::{
    SkillCallback, SkillCategory, SkillContext, types::{Skill, SkillParameter}
};

#[derive(Debug)]
pub struct AudioControlInputDeviceListSkill;

#[async_trait::async_trait]
impl Skill for AudioControlInputDeviceListSkill {
    fn name(&self) -> &str {
        "audio_control_input_device_list"
    }

    fn description(&self) -> &str {
        "List all available audio input devices (microphones)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see what microphone devices are available."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "audio_control_input_device_list"
        })
    }

    fn example_output(&self) -> String {
        "Found 2 input devices:\n1. Default Microphone (default)\n2. Microphone Array".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Audio
    }

    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let devices = list_input_devices()?;

        if devices.is_empty() {
            return Ok("No input devices found".to_string());
        }

        let mut result = format!("Found {} input devices:\n", devices.len());
        for (i, device) in devices.iter().enumerate() {
            let default_marker = if device.is_default { " (default)" } else { "" };
            result.push_str(&format!(
                "{}. {}{} (ID: {})\n",
                i + 1,
                device.name,
                default_marker,
                device.id
            ));
        }

        Ok(result)
    }
}
