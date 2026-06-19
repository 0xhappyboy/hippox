//! Bluetooth battery get skill - read device battery level

use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug)]
pub struct BluetoothBatteryGetDriver;

#[async_trait::async_trait]
impl Driver for BluetoothBatteryGetDriver {
    fn name(&self) -> &str {
        "bluetooth_battery_get"
    }

    fn description(&self) -> &str {
        "Get the battery level of a connected Bluetooth device (if supported)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check the battery level of Bluetooth headphones, speakers, or other devices that support the Battery Service."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the device".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_battery_get",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        })
    }

    fn example_output(&self) -> String {
        "Battery level: 75%".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Bluetooth
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
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
            for line in stdout.lines() {
                if line.contains("Battery") || line.contains("battery") {
                    if let Some(percentage) = line.split_whitespace().find(|w| w.contains("%")) {
                        return Ok(format!("Battery level: {}", percentage));
                    }
                }
            }
        }

        Ok(format!("Battery level not available for {}", mac_address))
    }
}
