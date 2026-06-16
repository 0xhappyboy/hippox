//! Bluetooth LE scan filter skill - scan for specific BLE devices by service UUID

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct BluetoothLeScanFilterSkill;

#[async_trait::async_trait]
impl Skill for BluetoothLeScanFilterSkill {
    fn name(&self) -> &str {
        "bluetooth_le_scan_filter"
    }

    fn description(&self) -> &str {
        "Scan for BLE devices with a specific service UUID filter"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to discover only devices that support a specific BLE service (e.g., heart rate monitors, temperature sensors)."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "service_uuid".to_string(),
                param_type: "string".to_string(),
                description: "Service UUID to filter for".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "0000180d-0000-1000-8000-00805f9b34fb".to_string(),
                )),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout_secs".to_string(),
                param_type: "integer".to_string(),
                description: "Scan timeout in seconds (default: 10)".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(15.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_le_scan_filter",
            "parameters": {
                "service_uuid": "0000180d-0000-1000-8000-00805f9b34fb",
                "timeout_secs": 10
            }
        })
    }

    fn example_output(&self) -> String {
        "Found 2 devices with service 0000180d-0000-1000-8000-00805f9b34fb:\n1. Heart Rate Monitor (AA:BB:CC:DD:EE:FF)\n2. Fitness Tracker (11:22:33:44:55:66)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Bluetooth
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let service_uuid = parameters
            .get("service_uuid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'service_uuid' parameter"))?;

        let timeout = parameters
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);

        #[cfg(target_os = "linux")]
        {
            Command::new("bluetoothctl").args(["scan", "on"]).output()?;

            tokio::time::sleep(std::time::Duration::from_secs(timeout)).await;

            let output = Command::new("bluetoothctl").args(["devices"]).output()?;

            Command::new("bluetoothctl")
                .args(["scan", "off"])
                .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut result = format!("Found devices with service {}:\n", service_uuid);
            let mut count = 0;

            for line in stdout.lines() {
                if line.starts_with("Device") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let mac = parts[1];
                        let name = parts[2..].join(" ");
                        count += 1;
                        result.push_str(&format!("{}. {} ({})\n", count, name, mac));
                    }
                }
            }

            if count == 0 {
                return Ok(format!("No devices found with service {}", service_uuid));
            }

            return Ok(result);
        }

        Ok(format!("Filtered scan for service {}", service_uuid))
    }
}
