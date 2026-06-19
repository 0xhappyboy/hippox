//! Bluetooth turn on skill - enable Bluetooth adapter

use super::common::bluetooth_on;
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
pub struct BluetoothTurnOnDriver;

#[async_trait::async_trait]
impl Driver for BluetoothTurnOnDriver {
    fn name(&self) -> &str {
        "bluetooth_turn_on"
    }

    fn description(&self) -> &str {
        "Turn on the Bluetooth adapter"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to enable Bluetooth. After turning on, you can scan for devices and pair with them."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_turn_on"
        })
    }

    fn example_output(&self) -> String {
        "Bluetooth turned on".to_string()
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
        bluetooth_on()?;
        Ok("Bluetooth turned on".to_string())
    }
}
