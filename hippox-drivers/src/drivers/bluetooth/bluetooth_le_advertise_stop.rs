//! Bluetooth LE advertise stop skill - stop BLE advertising

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
pub struct BluetoothLeAdvertiseStopDriver;

#[async_trait::async_trait]
impl Driver for BluetoothLeAdvertiseStopDriver {
    fn name(&self) -> &str {
        "bluetooth_le_advertise_stop"
    }

    fn description(&self) -> &str {
        "Stop BLE advertising"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to stop BLE broadcasting started by bluetooth_le_advertise_start."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_le_advertise_stop"
        })
    }

    fn example_output(&self) -> String {
        "BLE advertising stopped".to_string()
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
        #[cfg(target_os = "linux")]
        {
            Command::new("bluetoothctl")
                .args(["advertise", "off"])
                .output()?;
        }

        Ok("BLE advertising stopped".to_string())
    }
}
