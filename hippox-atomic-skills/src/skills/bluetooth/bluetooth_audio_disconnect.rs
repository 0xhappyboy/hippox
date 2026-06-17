//! Bluetooth audio disconnect skill - disconnect A2DP audio device

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
pub struct BluetoothAudioDisconnectSkill;

#[async_trait::async_trait]
impl Skill for BluetoothAudioDisconnectSkill {
    fn name(&self) -> &str {
        "bluetooth_audio_disconnect"
    }

    fn description(&self) -> &str {
        "Disconnect a Bluetooth audio device (headphones, speakers)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to disconnect Bluetooth headphones or speakers."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the audio device".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_audio_disconnect",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        })
    }

    fn example_output(&self) -> String {
        "Disconnected from audio device: AA:BB:CC:DD:EE:FF".to_string()
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

        #[cfg(target_os = "linux")]
        {
            Command::new("bluetoothctl")
                .args(["disconnect", mac_address])
                .output()?;
        }

        Ok(format!("Disconnected from audio device: {}", mac_address))
    }
}
