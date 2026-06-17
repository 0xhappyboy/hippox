//! Bluetooth unpair skill - unpair/remove a paired device

use super::common::unpair_device;
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
pub struct BluetoothUnpairSkill;

#[async_trait::async_trait]
impl Skill for BluetoothUnpairSkill {
    fn name(&self) -> &str {
        "bluetooth_unpair"
    }

    fn description(&self) -> &str {
        "Unpair/remove a paired Bluetooth device"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to remove a device from the paired devices list. The device will no longer automatically connect."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the device to unpair".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_unpair",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        })
    }

    fn example_output(&self) -> String {
        "Unpaired device: AA:BB:CC:DD:EE:FF".to_string()
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

        unpair_device(mac_address)?;

        Ok(format!("Unpaired device: {}", mac_address))
    }
}
