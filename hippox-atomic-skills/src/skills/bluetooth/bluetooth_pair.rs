//! Bluetooth pair skill - pair with a Bluetooth device

use super::common::pair_device;
use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct BluetoothPairSkill;

#[async_trait::async_trait]
impl Skill for BluetoothPairSkill {
    fn name(&self) -> &str {
        "bluetooth_pair"
    }

    fn description(&self) -> &str {
        "Pair with a Bluetooth device using its MAC address (may require PIN confirmation)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to pair with a discovered Bluetooth device. Make sure the device is in pairable mode."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the device to pair with (format: XX:XX:XX:XX:XX:XX)"
                    .to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "pin".to_string(),
                param_type: "string".to_string(),
                description: "PIN code for pairing (if required)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0000".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_pair",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        })
    }

    fn example_output(&self) -> String {
        "Paired with device: AA:BB:CC:DD:EE:FF".to_string()
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

        pair_device(mac_address)?;

        Ok(format!("Paired with device: {}", mac_address))
    }
}
