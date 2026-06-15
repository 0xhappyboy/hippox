//! Bluetooth device info skill - get detailed device information

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct BluetoothDeviceInfoSkill;

#[async_trait::async_trait]
impl Skill for BluetoothDeviceInfoSkill {
    fn name(&self) -> &str {
        "bluetooth_device_info"
    }

    fn description(&self) -> &str {
        "Get detailed information about a Bluetooth device (vendor, RSSI, supported services)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get detailed information like signal strength and manufacturer info about a device."
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_device_info",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        })
    }

    fn example_output(&self) -> String {
        "Device Info:\n- Name: My Headphones\n- MAC: AA:BB:CC:DD:EE:FF\n- RSSI: -45 dBm\n- Type: Audio\n- Paired: Yes\n- Connected: Yes".to_string()
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
            let output = Command::new("bluetoothctl")
                .args(["info", mac_address])
                .output()?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut result = format!("Device Info for {}:\n", mac_address);
            
            for line in stdout.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    result.push_str(&format!("- {}\n", line));
                }
            }
            
            return Ok(result);
        }
        
        Ok(format!("Device info for {}", mac_address))
    }
}