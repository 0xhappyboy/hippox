//! Bluetooth turn on skill - enable Bluetooth adapter
//!
//! This driver provides functionality to enable the Bluetooth adapter.
use super::common::bluetooth_on;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for turning on Bluetooth
///
/// This driver enables the Bluetooth adapter, allowing the system to
/// scan for devices and make connections.
#[derive(Debug)]
pub struct BluetoothTurnOnDriver;
#[async_trait::async_trait]
impl Driver for BluetoothTurnOnDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_turn_on"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Turn on the Bluetooth adapter"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to enable Bluetooth. After turning on, you can scan for devices and pair with them."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_turn_on"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Bluetooth turned on".to_string()
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        DriverCategory::Bluetooth
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing bluetooth_turn_on driver");
        bluetooth_on().map_err(|e| DriverError::execution(format!("Failed to turn on Bluetooth: {}", e)))?;
        info!("Bluetooth turned on");
        Ok("Bluetooth turned on".to_string())
    }
}
