// display_control/display_control_brightness_get.rs
//! Display brightness get skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_brightness;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct DisplayControlBrightnessGetSkill;

#[async_trait::async_trait]
impl Skill for DisplayControlBrightnessGetSkill {
    fn name(&self) -> &str {
        "display_control_brightness_get"
    }

    fn description(&self) -> &str {
        "Get the current display brightness level (laptops only)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the screen brightness (0-100). Works on laptops, may not work on desktops."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_brightness_get"
        })
    }

    fn example_output(&self) -> String {
        "Display brightness: 75%".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Display
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let brightness = get_brightness()?;

        Ok(format!("Display brightness: {}%", brightness))
    }
}
