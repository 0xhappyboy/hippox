//! Bluetooth connect skill - connect to a paired device

use super::common::connect_device;
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
pub struct BluetoothConnectDriver;

#[async_trait::async_trait]
impl Driver for BluetoothConnectDriver {
    fn name(&self) -> &str {
        "bluetooth_connect"
    }

    fn description(&self) -> &str {
        "Connect to a paired Bluetooth device (establish RFCOMM channel)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to connect to a device that is already paired. The device must be in range and powered on."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the device to connect to".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_connect",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        })
    }

    fn example_output(&self) -> String {
        "Connected to device: AA:BB:CC:DD:EE:FF".to_string()
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

        connect_device(mac_address)?;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        Ok(format!("Connected to device: {}", mac_address))
    }
}
