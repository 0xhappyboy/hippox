//! Bluetooth disconnect skill - disconnect a connected device
//!
//! This driver provides functionality to disconnect from a connected
//! Bluetooth device while keeping it paired.
use super::common::disconnect_device;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for disconnecting Bluetooth devices
///
/// This driver disconnects an active Bluetooth connection while keeping
/// the device paired for future connections.
#[derive(Debug)]
pub struct BluetoothDisconnectDriver;
#[async_trait::async_trait]
impl Driver for BluetoothDisconnectDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_disconnect"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Disconnect a connected Bluetooth device"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to disconnect an active Bluetooth connection. The device will remain paired."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the device to disconnect".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_disconnect",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Disconnected from device: AA:BB:CC:DD:EE:FF".to_string()
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        DriverCategory::Bluetooth
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing bluetooth_disconnect driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        debug!("Disconnecting from device: {}", mac_address);
        disconnect_device(mac_address).map_err(|e| DriverError::execution(format!("Failed to disconnect: {}", e)))?;
        info!("Disconnected from device: {}", mac_address);
        Ok(format!("Disconnected from device: {}", mac_address))
    }
}
