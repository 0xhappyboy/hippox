//! Bluetooth turn off skill - disable Bluetooth adapter
//!
//! This driver provides functionality to disable the Bluetooth adapter.
use super::common::bluetooth_off;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for turning off Bluetooth
///
/// This driver disables the Bluetooth adapter, disconnecting all
/// connected devices.
#[derive(Debug)]
pub struct BluetoothTurnOffDriver;
#[async_trait::async_trait]
impl Driver for BluetoothTurnOffDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_turn_off"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Turn off the Bluetooth adapter"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to disable Bluetooth. This will disconnect all connected devices."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_turn_off"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Bluetooth turned off".to_string()
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
        debug!("Executing bluetooth_turn_off driver");
        bluetooth_off().map_err(|e| DriverError::execution(format!("Failed to turn off Bluetooth: {}", e)))?;
        info!("Bluetooth turned off");
        Ok("Bluetooth turned off".to_string())
    }
}
