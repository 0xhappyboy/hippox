//! Bluetooth unpair skill - unpair/remove a paired device
//!
//! This driver provides functionality to unpair and remove a paired
//! Bluetooth device.
use super::common::unpair_device;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for unpairing Bluetooth devices
///
/// This driver removes a device from the paired devices list. The device
/// will no longer be able to connect automatically.
#[derive(Debug)]
pub struct BluetoothUnpairDriver;
#[async_trait::async_trait]
impl Driver for BluetoothUnpairDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_unpair"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Unpair/remove a paired Bluetooth device"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to remove a device from the paired devices list. The device will no longer automatically connect."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the device to unpair".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_unpair",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Unpaired device: AA:BB:CC:DD:EE:FF".to_string()
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
        debug!("Executing bluetooth_unpair driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        debug!("Unpairing device: {}", mac_address);
        unpair_device(mac_address).map_err(|e| DriverError::execution(format!("Failed to unpair device: {}", e)))?;
        info!("Unpaired device: {}", mac_address);
        Ok(format!("Unpaired device: {}", mac_address))
    }
}
