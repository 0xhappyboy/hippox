//! Bluetooth firmware version skill - get device firmware version

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
pub struct BluetoothFirmwareVersionDriver;

#[async_trait::async_trait]
impl Driver for BluetoothFirmwareVersionDriver {
    fn name(&self) -> &str {
        "bluetooth_firmware_version"
    }

    fn description(&self) -> &str {
        "Get the firmware version of a Bluetooth device (if available)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check the firmware version of your Bluetooth device."
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
            "action": "bluetooth_firmware_version",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        })
    }

    fn example_output(&self) -> String {
        "Firmware version: 1.2.3".to_string()
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
                if line.contains("Firmware") || line.contains("Version") {
                    if let Some(ver) = line.split(':').nth(1) {
                        return Ok(format!("Firmware version: {}", ver.trim()));
                    }
                }
            }
        }

        Ok(format!(
            "Firmware version not available for {}",
            mac_address
        ))
    }
}
