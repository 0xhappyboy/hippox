//! Bluetooth HID connect skill - connect to HID device (keyboard, mouse)

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
pub struct BluetoothHidConnectDriver;

#[async_trait::async_trait]
impl Driver for BluetoothHidConnectDriver {
    fn name(&self) -> &str {
        "bluetooth_hid_connect"
    }

    fn description(&self) -> &str {
        "Connect to a Bluetooth HID device (keyboard, mouse, gamepad)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to connect to Bluetooth input devices. The device must be paired first."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the HID device".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_hid_connect",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        })
    }

    fn example_output(&self) -> String {
        "Connected to HID device: AA:BB:CC:DD:EE:FF".to_string()
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
            Command::new("bluetoothctl")
                .args(["connect", mac_address])
                .output()?;
        }

        Ok(format!("Connected to HID device: {}", mac_address))
    }
}
