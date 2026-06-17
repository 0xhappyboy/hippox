//! Bluetooth BLE notify skill - subscribe to BLE notifications/indications

use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug)]
pub struct BluetoothBleNotifySkill;

#[async_trait::async_trait]
impl Skill for BluetoothBleNotifySkill {
    fn name(&self) -> &str {
        "bluetooth_ble_notify"
    }

    fn description(&self) -> &str {
        "Subscribe to notifications/indications from a BLE device characteristic"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to receive real-time updates from BLE devices like heart rate monitors or sensors."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the BLE device".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "characteristic_uuid".to_string(),
                param_type: "string".to_string(),
                description: "UUID of the characteristic to subscribe to".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "00002a37-0000-1000-8000-00805f9b34fb".to_string(),
                )),
                enum_values: None,
            },
            SkillParameter {
                name: "enable".to_string(),
                param_type: "boolean".to_string(),
                description: "Enable (true) or disable (false) notifications".to_string(),
                required: true,
                default: None,
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_ble_notify",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "characteristic_uuid": "00002a37-0000-1000-8000-00805f9b34fb",
                "enable": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Notifications enabled for characteristic".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Bluetooth
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let mac_address = parameters
            .get("mac_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'mac_address' parameter"))?;

        let characteristic_uuid = parameters
            .get("characteristic_uuid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'characteristic_uuid' parameter"))?;

        let enable = parameters
            .get("enable")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow::anyhow!("Missing 'enable' parameter"))?;

        let action = if enable { "notify" } else { "unnotify" };

        #[cfg(target_os = "linux")]
        {
            Command::new("bluetoothctl")
                .args([action, mac_address, characteristic_uuid])
                .output()?;
        }

        if enable {
            Ok("Notifications enabled for characteristic".to_string())
        } else {
            Ok("Notifications disabled for characteristic".to_string())
        }
    }
}
