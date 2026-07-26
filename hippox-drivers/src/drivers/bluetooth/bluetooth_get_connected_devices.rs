//! Bluetooth get connected devices skill - list currently connected devices
//!
//! This driver provides functionality to list all currently connected
//! Bluetooth devices.
use super::common::get_connected_devices;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing connected Bluetooth devices
///
/// This driver retrieves and displays the list of Bluetooth devices that
/// are currently connected to the system.
#[derive(Debug)]
pub struct BluetoothGetConnectedDevicesDriver;
#[async_trait::async_trait]
impl Driver for BluetoothGetConnectedDevicesDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_get_connected_devices"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "List currently connected Bluetooth devices (distinct from paired devices)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to see which Bluetooth devices are actively connected right now."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_get_connected_devices"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Connected devices:\n1. Headphones (AA:BB:CC:DD:EE:FF) - Audio\n2. Mouse (11:22:33:44:55:66) - HID".to_string()
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
        debug!("Executing bluetooth_get_connected_devices driver");
        let devices = get_connected_devices().map_err(|e| DriverError::execution(format!("Failed to get connected devices: {}", e)))?;
        if devices.is_empty() {
            debug!("No connected devices found");
            return Ok("No Bluetooth devices currently connected".to_string());
        }
        debug!("Found {} connected devices", devices.len());
        let mut result = format!("Connected devices:\n");
        for (i, device) in devices.iter().enumerate() {
            result.push_str(&format!(
                "{}. {} ({}){}\n",
                i + 1,
                device.name,
                device.mac_address,
                if !device.device_type.is_empty() { format!(" - {}", device.device_type) } else { String::new() }
            ));
        }
        info!("Listed {} connected devices", devices.len());
        Ok(result)
    }
}
