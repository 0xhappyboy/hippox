//! Bluetooth set device name skill - change the Bluetooth adapter name

use super::common::set_device_name;
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
pub struct BluetoothSetDeviceNameSkill;

#[async_trait::async_trait]
impl Skill for BluetoothSetDeviceNameSkill {
    fn name(&self) -> &str {
        "bluetooth_set_device_name"
    }

    fn description(&self) -> &str {
        "Change the Bluetooth adapter name that other devices see"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to customize how your device appears to other Bluetooth devices."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "name".to_string(),
            param_type: "string".to_string(),
            description: "New Bluetooth device name (max 248 characters)".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("My Computer".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_set_device_name",
            "parameters": {
                "name": "My Computer"
            }
        })
    }

    fn example_output(&self) -> String {
        "Bluetooth device name set to: My Computer".to_string()
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
        let name = parameters
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;
        if name.len() > 248 {
            anyhow::bail!("Device name must be 248 characters or less");
        }
        set_device_name(name)?;
        Ok(format!("Bluetooth device name set to: {}", name))
    }
}
