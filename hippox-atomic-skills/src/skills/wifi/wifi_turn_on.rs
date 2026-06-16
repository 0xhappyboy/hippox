//! WiFi turn on skill - enable WiFi adapter

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::wifi_on;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WifiTurnOnSkill;

#[async_trait::async_trait]
impl Skill for WifiTurnOnSkill {
    fn name(&self) -> &str {
        "wifi_turn_on"
    }

    fn description(&self) -> &str {
        "Turn on the WiFi adapter/radio"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to enable WiFi when it is turned off. After turning on, the device may automatically connect to known networks."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_turn_on"
        })
    }

    fn example_output(&self) -> String {
        "WiFi turned on".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Wifi
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        wifi_on()?;
        Ok("WiFi turned on".to_string())
    }
}
