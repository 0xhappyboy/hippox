//! Bluetooth pairable skill - set device to discoverable/pairable mode

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::set_discoverable;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct BluetoothPairableSkill;

#[async_trait::async_trait]
impl Skill for BluetoothPairableSkill {
    fn name(&self) -> &str {
        "bluetooth_pairable"
    }

    fn description(&self) -> &str {
        "Set the device to be discoverable/pairable mode so other devices can find it"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to make your device visible to other Bluetooth devices for pairing."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "enabled".to_string(),
                param_type: "boolean".to_string(),
                description: "Enable (true) or disable (false) discoverable mode".to_string(),
                required: true,
                default: None,
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout_secs".to_string(),
                param_type: "integer".to_string(),
                description: "How long to stay discoverable (default: 120 seconds)".to_string(),
                required: false,
                default: Some(Value::Number(120.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_pairable",
            "parameters": {
                "enabled": true,
                "timeout_secs": 120
            }
        })
    }

    fn example_output(&self) -> String {
        "Bluetooth discoverable mode enabled for 120 seconds".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Bluetooth
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let enabled = parameters
            .get("enabled")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow::anyhow!("Missing 'enabled' parameter"))?;

        let timeout = parameters
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .map(|t| t as u32);

        set_discoverable(enabled, timeout)?;

        if enabled {
            if let Some(t) = timeout {
                Ok(format!(
                    "Bluetooth discoverable mode enabled for {} seconds",
                    t
                ))
            } else {
                Ok("Bluetooth discoverable mode enabled".to_string())
            }
        } else {
            Ok("Bluetooth discoverable mode disabled".to_string())
        }
    }
}
