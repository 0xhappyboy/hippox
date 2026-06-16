//! Bluetooth get MAC address skill - get local Bluetooth MAC address

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_mac_address;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct BluetoothGetMacAddressSkill;

#[async_trait::async_trait]
impl Skill for BluetoothGetMacAddressSkill {
    fn name(&self) -> &str {
        "bluetooth_get_mac_address"
    }

    fn description(&self) -> &str {
        "Get the Bluetooth adapter's MAC address"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the hardware address of your Bluetooth adapter."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_get_mac_address"
        })
    }

    fn example_output(&self) -> String {
        "Bluetooth MAC Address: AA:BB:CC:DD:EE:FF".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Bluetooth
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let mac_address = get_mac_address()?;
        Ok(format!("Bluetooth MAC Address: {}", mac_address))
    }
}
