//! Bluetooth audio connect skill - connect to A2DP audio device

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct BluetoothAudioConnectSkill;

#[async_trait::async_trait]
impl Skill for BluetoothAudioConnectSkill {
    fn name(&self) -> &str {
        "bluetooth_audio_connect"
    }

    fn description(&self) -> &str {
        "Connect to a Bluetooth audio device (headphones, speakers) using A2DP profile"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to connect to Bluetooth headphones or speakers. The device must be paired first."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the audio device".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_audio_connect",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        })
    }

    fn example_output(&self) -> String {
        "Connected to audio device: AA:BB:CC:DD:EE:FF".to_string()
    }

    fn category(&self) -> &str {
        "bluetooth"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let mac_address = parameters
            .get("mac_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'mac_address' parameter"))?;
        
        #[cfg(target_os = "linux")]
        {
            Command::new("bluetoothctl")
                .args(["connect", mac_address])
                .output()?;
            
            // Set audio profile
            Command::new("pactl")
                .args(["set-card-profile", "bluez_card.0", "a2dp-sink"])
                .output()
                .ok();
        }
        
        Ok(format!("Connected to audio device: {}", mac_address))
    }
}