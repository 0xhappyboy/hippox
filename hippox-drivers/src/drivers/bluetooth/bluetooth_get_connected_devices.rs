//! Bluetooth get connected devices skill - list currently connected devices

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_connected_devices;
use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};

#[derive(Debug)]
pub struct BluetoothGetConnectedDevicesDriver;

#[async_trait::async_trait]
impl Driver for BluetoothGetConnectedDevicesDriver {
    fn name(&self) -> &str {
        "bluetooth_get_connected_devices"
    }

    fn description(&self) -> &str {
        "List currently connected Bluetooth devices (distinct from paired devices)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see which Bluetooth devices are actively connected right now."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
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

    fn category(&self) -> DriverCategory {
        DriverCategory::Bluetooth
    }

    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
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
