// display_control/display_control_orientation_get.rs
//! Display orientation get skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use super::common::get_orientation;

#[derive(Debug)]
pub struct DisplayControlOrientationGetSkill;

#[async_trait::async_trait]
impl Skill for DisplayControlOrientationGetSkill {
    fn name(&self) -> &str {
        "display_control_orientation_get"
    }

    fn description(&self) -> &str {
        "Get the display orientation (landscape, portrait, etc.)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check if the display is in landscape or portrait mode."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_orientation_get"
        })
    }

    fn example_output(&self) -> String {
        "Display orientation: landscape".to_string()
    }

    fn category(&self) -> &str {
        "display_control"
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let orientation = get_orientation(None)?;
        
        Ok(format!("Display orientation: {}", orientation))
    }
}