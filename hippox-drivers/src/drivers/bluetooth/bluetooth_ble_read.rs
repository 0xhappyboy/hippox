//! Bluetooth BLE read skill - read characteristic values from BLE device

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
pub struct BluetoothBleReadDriver;

#[async_trait::async_trait]
impl Driver for BluetoothBleReadDriver {
    fn name(&self) -> &str {
        "bluetooth_ble_read"
    }

    fn description(&self) -> &str {
        "Read a characteristic value from a BLE device"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to read sensor data or device state from a BLE device. Requires the characteristic UUID."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the BLE device".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "characteristic_uuid".to_string(),
                param_type: "string".to_string(),
                description: "UUID of the characteristic to read".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "00002a19-0000-1000-8000-00805f9b34fb".to_string(),
                )),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_ble_read",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "characteristic_uuid": "00002a19-0000-1000-8000-00805f9b34fb"
            }
        })
    }

    fn example_output(&self) -> String {
        "Characteristic value: 0x64 (100)".to_string()
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

        let characteristic_uuid = parameters
            .get("characteristic_uuid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'characteristic_uuid' parameter"))?;

        #[cfg(target_os = "linux")]
        {
            let output = Command::new("bluetoothctl")
                .args(["get-value", mac_address, characteristic_uuid])
                .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            return Ok(format!("Characteristic value: {}", stdout.trim()));
        }

        Ok(format!(
            "Read from {} characteristic {}",
            mac_address, characteristic_uuid
        ))
    }
}
