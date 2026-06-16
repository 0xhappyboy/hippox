//! Bluetooth auto connect toggle skill - enable/disable auto reconnect

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct BluetoothAutoConnectToggleSkill;

#[async_trait::async_trait]
impl Skill for BluetoothAutoConnectToggleSkill {
    fn name(&self) -> &str {
        "bluetooth_auto_connect_toggle"
    }

    fn description(&self) -> &str {
        "Enable or disable auto-reconnect for a paired Bluetooth device"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to control whether a device automatically reconnects when it comes back into range."
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
                name: "enabled".to_string(),
                param_type: "boolean".to_string(),
                description: "Enable (true) or disable (false) auto-connect".to_string(),
                required: true,
                default: None,
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_auto_connect_toggle",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "enabled": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Auto-connect enabled for AA:BB:CC:DD:EE:FF".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Bluetooth
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let mac_address = parameters
            .get("mac_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'mac_address' parameter"))?;

        let enabled = parameters
            .get("enabled")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow::anyhow!("Missing 'enabled' parameter"))?;

        #[cfg(target_os = "linux")]
        {
            if enabled {
                Command::new("bluetoothctl")
                    .args(["trust", mac_address])
                    .output()?;
            } else {
                Command::new("bluetoothctl")
                    .args(["untrust", mac_address])
                    .output()?;
            }
        }

        if enabled {
            Ok(format!("Auto-connect enabled for {}", mac_address))
        } else {
            Ok(format!("Auto-connect disabled for {}", mac_address))
        }
    }
}
