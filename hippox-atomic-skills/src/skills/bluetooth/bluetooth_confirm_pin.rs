//! Bluetooth confirm PIN skill - confirm PIN code for pairing

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct BluetoothConfirmPinSkill;

#[async_trait::async_trait]
impl Skill for BluetoothConfirmPinSkill {
    fn name(&self) -> &str {
        "bluetooth_confirm_pin"
    }

    fn description(&self) -> &str {
        "Confirm a PIN code to complete Bluetooth pairing"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to respond to a pairing PIN request when a device shows a PIN code."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "pin".to_string(),
            param_type: "string".to_string(),
            description: "PIN code to confirm (usually 4-6 digits)".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("0000".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_confirm_pin",
            "parameters": {
                "pin": "123456"
            }
        })
    }

    fn example_output(&self) -> String {
        "PIN code 123456 confirmed".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Bluetooth
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let pin = parameters
            .get("pin")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'pin' parameter"))?;

        #[cfg(target_os = "linux")]
        {
            Command::new("bluetoothctl").args(["pin", pin]).output()?;
        }

        Ok(format!("PIN code {} confirmed", pin))
    }
}
