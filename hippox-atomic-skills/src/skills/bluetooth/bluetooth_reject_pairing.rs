//! Bluetooth reject pairing skill - reject incoming pairing request

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct BluetoothRejectPairingSkill;

#[async_trait::async_trait]
impl Skill for BluetoothRejectPairingSkill {
    fn name(&self) -> &str {
        "bluetooth_reject_pairing"
    }

    fn description(&self) -> &str {
        "Reject an incoming Bluetooth pairing request from another device"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to deny a pairing request from a device."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the device requesting pairing".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_reject_pairing",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        })
    }

    fn example_output(&self) -> String {
        "Pairing request rejected for AA:BB:CC:DD:EE:FF".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Bluetooth
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let mac_address = parameters
            .get("mac_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'mac_address' parameter"))?;

        #[cfg(target_os = "linux")]
        {
            Command::new("bluetoothctl")
                .args(["reject", mac_address])
                .output()?;
        }

        Ok(format!("Pairing request rejected for {}", mac_address))
    }
}
