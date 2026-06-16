//! Bluetooth scan skill - scan for nearby Bluetooth devices

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::scan_devices;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct BluetoothScanSkill;

#[async_trait::async_trait]
impl Skill for BluetoothScanSkill {
    fn name(&self) -> &str {
        "bluetooth_scan"
    }

    fn description(&self) -> &str {
        "Scan for nearby Bluetooth devices and return their names, MAC addresses, and types"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to discover Bluetooth devices in range. Scanning may take 5-10 seconds."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "timeout_secs".to_string(),
            param_type: "integer".to_string(),
            description: "Scan timeout in seconds (default: 10)".to_string(),
            required: false,
            default: Some(Value::Number(10.into())),
            example: Some(Value::Number(15.into())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_scan",
            "parameters": {
                "timeout_secs": 10
            }
        })
    }

    fn example_output(&self) -> String {
        "Found 3 devices:\n1. My Headphones (AA:BB:CC:DD:EE:FF) [Audio]\n2. My Phone (11:22:33:44:55:66) [Phone]\n3. Mouse (77:88:99:AA:BB:CC) [HID]".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Bluetooth
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let timeout = parameters
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);

        tokio::time::sleep(std::time::Duration::from_secs(timeout)).await;

        let devices = scan_devices()?;

        if devices.is_empty() {
            return Ok("No Bluetooth devices found".to_string());
        }

        let mut result = format!("Found {} devices:\n", devices.len());
        for (i, device) in devices.iter().enumerate() {
            let paired_marker = if device.paired { " [PAIRED]" } else { "" };
            let connected_marker = if device.connected { " [CONNECTED]" } else { "" };
            result.push_str(&format!(
                "{}. {}{}{} ({})",
                i + 1,
                device.name,
                paired_marker,
                connected_marker,
                device.mac_address
            ));
            if let Some(rssi) = device.rssi {
                result.push_str(&format!(" (Signal: {} dBm)", rssi));
            }
            result.push('\n');
        }

        Ok(result)
    }
}
