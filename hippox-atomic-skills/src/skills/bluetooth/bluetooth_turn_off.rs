//! Bluetooth turn off skill - disable Bluetooth adapter

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use super::common::bluetooth_off;

#[derive(Debug)]
pub struct BluetoothTurnOffSkill;

#[async_trait::async_trait]
impl Skill for BluetoothTurnOffSkill {
    fn name(&self) -> &str {
        "bluetooth_turn_off"
    }

    fn description(&self) -> &str {
        "Turn off the Bluetooth adapter"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to disable Bluetooth. This will disconnect all connected devices."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_turn_off"
        })
    }

    fn example_output(&self) -> String {
        "Bluetooth turned off".to_string()
    }

    fn category(&self) -> &str {
        "bluetooth"
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        bluetooth_off()?;
        Ok("Bluetooth turned off".to_string())
    }
}