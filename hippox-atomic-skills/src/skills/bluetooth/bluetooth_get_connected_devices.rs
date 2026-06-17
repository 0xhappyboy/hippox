//! Bluetooth get connected devices skill - list currently connected devices

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_connected_devices;
use crate::{
    SkillCallback, SkillCategory, SkillContext, types::{Skill, SkillParameter}
};

#[derive(Debug)]
pub struct BluetoothGetConnectedDevicesSkill;

#[async_trait::async_trait]
impl Skill for BluetoothGetConnectedDevicesSkill {
    fn name(&self) -> &str {
        "bluetooth_get_connected_devices"
    }

    fn description(&self) -> &str {
        "List currently connected Bluetooth devices (distinct from paired devices)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see which Bluetooth devices are actively connected right now."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_get_connected_devices"
        })
    }

    fn example_output(&self) -> String {
        "Connected devices:\n1. Headphones (AA:BB:CC:DD:EE:FF) - Audio\n2. Mouse (11:22:33:44:55:66) - HID".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Bluetooth
    }

    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let devices = get_connected_devices()?;
        if devices.is_empty() {
            return Ok("No Bluetooth devices currently connected".to_string());
        }
        let mut result = format!("Connected devices:\n");
        for (i, device) in devices.iter().enumerate() {
            result.push_str(&format!(
                "{}. {} ({}){}\n",
                i + 1,
                device.name,
                device.mac_address,
                if !device.device_type.is_empty() {
                    format!(" - {}", device.device_type)
                } else {
                    String::new()
                }
            ));
        }
        Ok(result)
    }
}
