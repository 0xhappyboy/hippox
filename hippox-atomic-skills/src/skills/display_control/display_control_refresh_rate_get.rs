// display_control/display_control_refresh_rate_get.rs
//! Display refresh rate get skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_refresh_rate;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct DisplayControlRefreshRateGetSkill;

#[async_trait::async_trait]
impl Skill for DisplayControlRefreshRateGetSkill {
    fn name(&self) -> &str {
        "display_control_refresh_rate_get"
    }

    fn description(&self) -> &str {
        "Get the current display refresh rate in Hz"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the monitor's refresh rate (e.g., 60Hz, 144Hz)."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_refresh_rate_get"
        })
    }

    fn example_output(&self) -> String {
        "Display refresh rate: 60 Hz".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Display
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let rate = get_refresh_rate(None)?;

        Ok(format!("Display refresh rate: {} Hz", rate))
    }
}
