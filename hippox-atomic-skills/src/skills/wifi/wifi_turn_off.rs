//! WiFi turn off skill - disable WiFi adapter

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::wifi_off;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WifiTurnOffSkill;

#[async_trait::async_trait]
impl Skill for WifiTurnOffSkill {
    fn name(&self) -> &str {
        "wifi_turn_off"
    }

    fn description(&self) -> &str {
        "Turn off the WiFi adapter/radio"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to disable WiFi. This will disconnect from any current network and turn off the radio."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_turn_off"
        })
    }

    fn example_output(&self) -> String {
        "WiFi turned off".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Wifi
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        wifi_off()?;
        Ok("WiFi turned off".to_string())
    }
}
