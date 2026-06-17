//! WiFi turn on skill - enable WiFi adapter

use super::common::wifi_on;
use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

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

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        wifi_on()?;
        Ok("WiFi turned on".to_string())
    }
}
