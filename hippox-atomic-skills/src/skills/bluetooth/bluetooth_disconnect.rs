//! Bluetooth disconnect skill - disconnect a connected device

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use super::common::disconnect_device;

#[derive(Debug)]
pub struct BluetoothDisconnectSkill;

#[async_trait::async_trait]
impl Skill for BluetoothDisconnectSkill {
    fn name(&self) -> &str {
        "bluetooth_disconnect"
    }

    fn description(&self) -> &str {
        "Disconnect a connected Bluetooth device"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to disconnect an active Bluetooth connection. The device will remain paired."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the device to disconnect".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_disconnect",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        })
    }

    fn example_output(&self) -> String {
        "Disconnected from device: AA:BB:CC:DD:EE:FF".to_string()
    }

    fn category(&self) -> &str {
        "bluetooth"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let mac_address = parameters
            .get("mac_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'mac_address' parameter"))?;
        
        disconnect_device(mac_address)?;
        
        Ok(format!("Disconnected from device: {}", mac_address))
    }
}