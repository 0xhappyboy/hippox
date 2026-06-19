//! Bluetooth get MAC address skill - get local Bluetooth MAC address

use super::common::get_mac_address;
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
pub struct BluetoothGetMacAddressDriver;

#[async_trait::async_trait]
impl Driver for BluetoothGetMacAddressDriver {
    fn name(&self) -> &str {
        "bluetooth_get_mac_address"
    }

    fn description(&self) -> &str {
        "Get the Bluetooth adapter's MAC address"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the hardware address of your Bluetooth adapter."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_get_mac_address"
        })
    }

    fn example_output(&self) -> String {
        "Bluetooth MAC Address: AA:BB:CC:DD:EE:FF".to_string()
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
        let mac_address = get_mac_address()?;
        Ok(format!("Bluetooth MAC Address: {}", mac_address))
    }
}
