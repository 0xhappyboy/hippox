//! Bluetooth list paired skill - list all paired devices
//!
//! This driver provides functionality to list all paired Bluetooth devices.
use super::common::list_paired_devices;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing paired Bluetooth devices
///
/// This driver retrieves and displays the list of all devices that have
/// been paired with the system.
#[derive(Debug)]
pub struct BluetoothListPairedDriver;
#[async_trait::async_trait]
impl Driver for BluetoothListPairedDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_list_paired"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "List all paired Bluetooth devices"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to see which Bluetooth devices have been paired with this computer."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_list_paired"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Found 2 paired devices:\n1. My Headphones (AA:BB:CC:DD:EE:FF) [CONNECTED]\n2. My Phone (11:22:33:44:55:66)".to_string()
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
        debug!("Executing bluetooth_list_paired driver");
        let devices = list_paired_devices().map_err(|e| DriverError::execution(format!("Failed to list paired devices: {}", e)))?;
        if devices.is_empty() {
            debug!("No paired devices found");
            return Ok("No paired devices found".to_string());
        }
        debug!("Found {} paired devices", devices.len());
        let mut result = format!("Found {} paired devices:\n", devices.len());
        for (i, device) in devices.iter().enumerate() {
            let connected_marker = if device.connected { " [CONNECTED]" } else { "" };
            result.push_str(&format!("{}. {} ({}){}\n", i + 1, device.name, device.mac_address, connected_marker));
        }
        info!("Listed {} paired devices", devices.len());
        Ok(result)
    }
}
