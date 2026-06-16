// display_control/display_control_primary_get.rs
//! Display primary get skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_primary_display;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct DisplayControlPrimaryGetSkill;

#[async_trait::async_trait]
impl Skill for DisplayControlPrimaryGetSkill {
    fn name(&self) -> &str {
        "display_control_primary_get"
    }

    fn description(&self) -> &str {
        "Get information about the primary display"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get details about the main monitor."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_primary_get"
        })
    }

    fn example_output(&self) -> String {
        "Primary display: Primary Display (1920x1080, 60Hz)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Display
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let display = get_primary_display()?;

        Ok(format!(
            "Primary display: {} ({}x{} @ {}Hz)",
            display.name, display.width, display.height, display.refresh_rate
        ))
    }
}
