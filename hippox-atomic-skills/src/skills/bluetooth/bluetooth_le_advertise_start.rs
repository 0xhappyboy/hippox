//! Bluetooth LE advertise start skill - start BLE advertising

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct BluetoothLeAdvertiseStartSkill;

#[async_trait::async_trait]
impl Skill for BluetoothLeAdvertiseStartSkill {
    fn name(&self) -> &str {
        "bluetooth_le_advertise_start"
    }

    fn description(&self) -> &str {
        "Start BLE advertising to make the device discoverable to BLE scanners"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to broadcast your device's presence to BLE devices. Useful for IoT and sensor applications."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "service_uuid".to_string(),
                param_type: "string".to_string(),
                description: "UUID of the service to advertise (optional)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0000180f-0000-1000-8000-00805f9b34fb".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "manufacturer_data".to_string(),
                param_type: "string".to_string(),
                description: "Manufacturer specific data (hex format)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0x01020304".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_le_advertise_start",
            "parameters": {
                "service_uuid": "0000180f-0000-1000-8000-00805f9b34fb"
            }
        })
    }

    fn example_output(&self) -> String {
        "BLE advertising started".to_string()
    }

    fn category(&self) -> &str {
        "bluetooth"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let service_uuid = parameters
            .get("service_uuid")
            .and_then(|v| v.as_str());
        
        #[cfg(target_os = "linux")]
        {
            Command::new("bluetoothctl")
                .args(["advertise", "on"])
                .output()?;
            
            if let Some(uuid) = service_uuid {
                Command::new("bluetoothctl")
                    .args(["advertise", "service", uuid])
                    .output()?;
            }
        }
        
        Ok("BLE advertising started".to_string())
    }
}