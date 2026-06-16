//! Bluetooth turn on skill - enable Bluetooth adapter

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::bluetooth_on;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct BluetoothTurnOnSkill;

#[async_trait::async_trait]
impl Skill for BluetoothTurnOnSkill {
    fn name(&self) -> &str {
        "bluetooth_turn_on"
    }

    fn description(&self) -> &str {
        "Turn on the Bluetooth adapter"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to enable Bluetooth. After turning on, you can scan for devices and pair with them."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_turn_on"
        })
    }

    fn example_output(&self) -> String {
        "Bluetooth turned on".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Bluetooth
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        bluetooth_on()?;
        Ok("Bluetooth turned on".to_string())
    }
}
