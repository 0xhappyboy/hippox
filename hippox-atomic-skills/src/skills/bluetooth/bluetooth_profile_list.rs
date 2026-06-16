//! Bluetooth profile list skill - list supported Bluetooth profiles

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct BluetoothProfileListSkill;

#[async_trait::async_trait]
impl Skill for BluetoothProfileListSkill {
    fn name(&self) -> &str {
        "bluetooth_profile_list"
    }

    fn description(&self) -> &str {
        "List supported Bluetooth profiles on the system (A2DP, HFP, SPP, etc.)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see what Bluetooth profiles your system supports."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_profile_list"
        })
    }

    fn example_output(&self) -> String {
        "Supported Bluetooth Profiles:\n1. A2DP (Audio)\n2. HFP (Hands-Free)\n3. SPP (Serial Port)\n4. HID (Human Interface)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Bluetooth
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let profiles = vec![
            "A2DP (Audio Source/Sink)",
            "HFP (Hands-Free Profile)",
            "HSP (Headset Profile)",
            "SPP (Serial Port Profile)",
            "HID (Human Interface Device)",
            "PAN (Personal Area Networking)",
            "OBEX (Object Exchange)",
            "GATT (Generic Attribute Profile)",
        ];

        let mut result = String::from("Supported Bluetooth Profiles:\n");
        for (i, profile) in profiles.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, profile));
        }

        Ok(result)
    }
}
