//! Bluetooth send file skill - send file via OBEX push

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;

use crate::types::{Skill, SkillParameter};
use super::common::send_file;

#[derive(Debug)]
pub struct BluetoothSendFileSkill;

#[async_trait::async_trait]
impl Skill for BluetoothSendFileSkill {
    fn name(&self) -> &str {
        "bluetooth_send_file"
    }

    fn description(&self) -> &str {
        "Send a file to a Bluetooth device via OBEX Object Push"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to send files (photos, documents, etc.) to a paired Bluetooth device like a phone."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the target device".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "file_path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the file to send".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/home/user/photo.jpg".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_send_file",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "file_path": "/home/user/photo.jpg"
            }
        })
    }

    fn example_output(&self) -> String {
        "File sent successfully to AA:BB:CC:DD:EE:FF".to_string()
    }

    fn category(&self) -> &str {
        "bluetooth"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let mac_address = parameters
            .get("mac_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'mac_address' parameter"))?;
        
        let file_path = parameters
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'file_path' parameter"))?;
        
        if !Path::new(file_path).exists() {
            anyhow::bail!("File does not exist: {}", file_path);
        }
        
        send_file(mac_address, file_path)?;
        
        Ok(format!("File sent successfully to {}", mac_address))
    }
}