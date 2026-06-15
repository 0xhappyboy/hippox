//! Bluetooth scan services skill - scan device services and characteristics (BLE GATT)

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct BluetoothScanServicesSkill;

#[async_trait::async_trait]
impl Skill for BluetoothScanServicesSkill {
    fn name(&self) -> &str {
        "bluetooth_scan_services"
    }

    fn description(&self) -> &str {
        "Scan and list available services and characteristics on a Bluetooth device (BLE GATT)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to discover what services a BLE device offers, useful for IoT and sensor devices."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the Bluetooth device".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_scan_services",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        })
    }

    fn example_output(&self) -> String {
        "Found 3 services:\n1. 00001800-0000-1000-8000-00805f9b34fb (Device Information)\n2. 0000180f-0000-1000-8000-00805f9b34fb (Battery Service)\n3. 0000180a-0000-1000-8000-00805f9b34fb (Device Name)".to_string()
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
                .args(["services", mac_address])
                .output()?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut result = String::new();
            let mut service_count = 0;
            
            for line in stdout.lines() {
                if line.contains("Service") || line.contains("UUID") {
                    service_count += 1;
                    result.push_str(&format!("{}. {}\n", service_count, line.trim()));
                }
            }
            
            if service_count == 0 {
                return Ok(format!("No services found for device {}", mac_address));
            }
            
            return Ok(format!("Found {} services:\n{}", service_count, result));
        }
        
        Ok(format!("Service scan for {}", mac_address))
    }
}