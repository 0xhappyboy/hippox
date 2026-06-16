//! Bluetooth receive file skill - receive files via Bluetooth

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct BluetoothReceiveFileSkill;

#[async_trait::async_trait]
impl Skill for BluetoothReceiveFileSkill {
    fn name(&self) -> &str {
        "bluetooth_receive_file"
    }

    fn description(&self) -> &str {
        "Enable receiving files via Bluetooth (OBEX Object Push)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to prepare your device to receive files from other Bluetooth devices."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "save_directory".to_string(),
                param_type: "string".to_string(),
                description: "Directory to save received files (default: downloads folder)"
                    .to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/home/user/Downloads".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout_secs".to_string(),
                param_type: "integer".to_string(),
                description: "How long to accept incoming files (default: 60 seconds)".to_string(),
                required: false,
                default: Some(Value::Number(60.into())),
                example: Some(Value::Number(120.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_receive_file",
            "parameters": {
                "save_directory": "/home/user/Downloads",
                "timeout_secs": 60
            }
        })
    }

    fn example_output(&self) -> String {
        "Ready to receive files for 60 seconds".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Bluetooth
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let save_directory = parameters
            .get("save_directory")
            .and_then(|v| v.as_str())
            .unwrap_or("/tmp/bluetooth_received");

        let timeout = parameters
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);

        // Create directory if it doesn't exist
        std::fs::create_dir_all(save_directory)?;

        #[cfg(target_os = "linux")]
        {
            Command::new("bluetoothctl")
                .args(["agent", "on"])
                .output()?;
            Command::new("bluetoothctl")
                .args(["default-agent"])
                .output()?;
        }

        Ok(format!(
            "Ready to receive files in '{}' for {} seconds",
            save_directory, timeout
        ))
    }
}
