// display_control/display_control_scale_get.rs
//! Display scale get skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_scale;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct DisplayControlScaleGetSkill;

#[async_trait::async_trait]
impl Skill for DisplayControlScaleGetSkill {
    fn name(&self) -> &str {
        "display_control_scale_get"
    }

    fn description(&self) -> &str {
        "Get the display scaling factor"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the DPI scaling factor (e.g., 1.0 for 100%, 1.5 for 150%)."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_scale_get"
        })
    }

    fn example_output(&self) -> String {
        "Display scale: 1.5x".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Display
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let scale = get_scale(None)?;

        Ok(format!("Display scale: {:.1}x", scale))
    }
}
