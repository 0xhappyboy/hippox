//! Bluetooth BLE write skill - write characteristic values to BLE device

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct BluetoothBleWriteSkill;

#[async_trait::async_trait]
impl Skill for BluetoothBleWriteSkill {
    fn name(&self) -> &str {
        "bluetooth_ble_write"
    }

    fn description(&self) -> &str {
        "Write a characteristic value to a BLE device"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to control BLE devices like lights, locks, or sensors. Requires the characteristic UUID."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the BLE device".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "characteristic_uuid".to_string(),
                param_type: "string".to_string(),
                description: "UUID of the characteristic to write".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "00002a19-0000-1000-8000-00805f9b34fb".to_string(),
                )),
                enum_values: None,
            },
            SkillParameter {
                name: "value".to_string(),
                param_type: "string".to_string(),
                description: "Value to write (hex format like '0x01' or string)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("0x01".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_ble_write",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "characteristic_uuid": "00002a19-0000-1000-8000-00805f9b34fb",
                "value": "0x01"
            }
        })
    }

    fn example_output(&self) -> String {
        "Characteristic value written successfully".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Bluetooth
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let mac_address = parameters
            .get("mac_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'mac_address' parameter"))?;

        let characteristic_uuid = parameters
            .get("characteristic_uuid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'characteristic_uuid' parameter"))?;

        let value = parameters
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'value' parameter"))?;

        #[cfg(target_os = "linux")]
        {
            Command::new("bluetoothctl")
                .args(["set-value", mac_address, characteristic_uuid, value])
                .output()?;
        }

        Ok("Characteristic value written successfully".to_string())
    }
}
