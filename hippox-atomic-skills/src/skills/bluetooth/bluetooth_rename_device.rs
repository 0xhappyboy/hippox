//! Bluetooth rename device skill - change display name of a paired device

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct BluetoothRenameDeviceSkill;

#[async_trait::async_trait]
impl Skill for BluetoothRenameDeviceSkill {
    fn name(&self) -> &str {
        "bluetooth_rename_device"
    }

    fn description(&self) -> &str {
        "Change the display name of a paired Bluetooth device"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to give a custom name to your Bluetooth devices for easier identification."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the device".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "New name for the device".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("My Headphones".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_rename_device",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "name": "My Headphones"
            }
        })
    }

    fn example_output(&self) -> String {
        "Device AA:BB:CC:DD:EE:FF renamed to 'My Headphones'".to_string()
    }

    fn category(&self) -> &str {
        "bluetooth"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let mac_address = parameters
            .get("mac_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'mac_address' parameter"))?;
        
        let name = parameters
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;
        
        #[cfg(target_os = "linux")]
        {
            Command::new("bluetoothctl")
                .args(["set-alias", mac_address, name])
                .output()?;
        }
        
        Ok(format!("Device {} renamed to '{}'", mac_address, name))
    }
}