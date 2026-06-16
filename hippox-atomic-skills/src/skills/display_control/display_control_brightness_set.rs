// display_control/display_control_brightness_set.rs
//! Display brightness set skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::set_brightness;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct DisplayControlBrightnessSetSkill;

#[async_trait::async_trait]
impl Skill for DisplayControlBrightnessSetSkill {
    fn name(&self) -> &str {
        "display_control_brightness_set"
    }

    fn description(&self) -> &str {
        "Set the display brightness level (laptops only)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to adjust screen brightness (0-100). Works on laptops, may not work on desktops."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "brightness".to_string(),
            param_type: "integer".to_string(),
            description: "Brightness level from 0 to 100".to_string(),
            required: true,
            default: None,
            example: Some(Value::Number(50.into())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_brightness_set",
            "parameters": {
                "brightness": 50
            }
        })
    }

    fn example_output(&self) -> String {
        "Display brightness set to 50%".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Display
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let brightness = parameters
            .get("brightness")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'brightness' parameter"))?
            as u32;
        let brightness = brightness.clamp(0, 100);
        set_brightness(brightness)?;
        Ok(format!("Display brightness set to {}%", brightness))
    }
}
