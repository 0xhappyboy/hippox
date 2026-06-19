//! Bluetooth turn off skill - disable Bluetooth adapter

use super::common::bluetooth_off;
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
pub struct BluetoothTurnOffDriver;

#[async_trait::async_trait]
impl Driver for BluetoothTurnOffDriver {
    fn name(&self) -> &str {
        "bluetooth_turn_off"
    }

    fn description(&self) -> &str {
        "Turn off the Bluetooth adapter"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to disable Bluetooth. This will disconnect all connected devices."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_turn_off"
        })
    }

    fn example_output(&self) -> String {
        "Bluetooth turned off".to_string()
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
        bluetooth_off()?;
        Ok("Bluetooth turned off".to_string())
    }
}
