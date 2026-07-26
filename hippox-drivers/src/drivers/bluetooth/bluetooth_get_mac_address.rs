//! Bluetooth get MAC address skill - get local Bluetooth MAC address
//!
//! This driver provides functionality to retrieve the local Bluetooth
//! adapter's MAC address.
use super::common::get_mac_address;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting the local Bluetooth MAC address
///
/// This driver retrieves the hardware MAC address of the system's
/// Bluetooth adapter.
#[derive(Debug)]
pub struct BluetoothGetMacAddressDriver;
#[async_trait::async_trait]
impl Driver for BluetoothGetMacAddressDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_get_mac_address"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the Bluetooth adapter's MAC address"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the hardware address of your Bluetooth adapter."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_get_mac_address"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Bluetooth MAC Address: AA:BB:CC:DD:EE:FF".to_string()
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
        debug!("Executing bluetooth_get_mac_address driver");
        let mac_address = get_mac_address().map_err(|e| DriverError::execution(format!("Failed to get MAC address: {}", e)))?;
        info!("Bluetooth MAC address retrieved: {}", mac_address);
        return Ok(format!("Bluetooth MAC Address: {}", mac_address));
    }
}
