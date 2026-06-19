//! Bluetooth list paired skill - list all paired devices

use super::common::list_paired_devices;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct BluetoothListPairedDriver;

#[async_trait::async_trait]
impl Driver for BluetoothListPairedDriver {
    fn name(&self) -> &str {
        "bluetooth_list_paired"
    }

    fn description(&self) -> &str {
        "List all paired Bluetooth devices"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see which Bluetooth devices have been paired with this computer."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_list_paired"
        })
    }

    fn example_output(&self) -> String {
        "Found 2 paired devices:\n1. My Headphones (AA:BB:CC:DD:EE:FF) [Connected]\n2. My Phone (11:22:33:44:55:66)".to_string()
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
        let devices = list_paired_devices()?;

        if devices.is_empty() {
            return Ok("No paired devices found".to_string());
        }

        let mut result = format!("Found {} paired devices:\n", devices.len());
        for (i, device) in devices.iter().enumerate() {
            let connected_marker = if device.connected { " [CONNECTED]" } else { "" };
            result.push_str(&format!(
                "{}. {} ({}){}\n",
                i + 1,
                device.name,
                device.mac_address,
                connected_marker
            ));
        }

        Ok(result)
    }
}
