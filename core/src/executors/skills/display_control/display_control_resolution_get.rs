// display_control/display_control_resolution_get.rs
//! Display resolution get skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::types::{Skill, SkillParameter};
use super::common::get_resolution;

#[derive(Debug)]
pub struct DisplayControlResolutionGetSkill;

#[async_trait::async_trait]
impl Skill for DisplayControlResolutionGetSkill {
    fn name(&self) -> &str {
        "display_control_resolution_get"
    }

    fn description(&self) -> &str {
        "Get the current display resolution"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the width and height of the primary display."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_resolution_get"
        })
    }

    fn example_output(&self) -> String {
        "Current resolution: 1920x1080".to_string()
    }

    fn category(&self) -> &str {
        "display_control"
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let (width, height) = get_resolution(None)?;
        
        Ok(format!("Current resolution: {}x{}", width, height))
    }
}