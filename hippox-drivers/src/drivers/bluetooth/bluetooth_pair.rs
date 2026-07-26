//! Bluetooth pair skill - pair with a Bluetooth device
//!
//! This driver provides functionality to initiate pairing with a Bluetooth device.
use super::common::pair_device;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for pairing with Bluetooth devices
///
/// This driver initiates pairing with a discovered Bluetooth device.
/// The device must be in pairable mode.
#[derive(Debug)]
pub struct BluetoothPairDriver;
#[async_trait::async_trait]
impl Driver for BluetoothPairDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_pair"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Pair with a Bluetooth device using its MAC address (may require PIN confirmation)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to pair with a discovered Bluetooth device. Make sure the device is in pairable mode."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the device to pair with (format: XX:XX:XX:XX:XX:XX)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "pin".to_string(),
                param_type: "string".to_string(),
                description: "PIN code for pairing (if required)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0000".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_pair",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Paired with device: AA:BB:CC:DD:EE:FF".to_string()
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
        debug!("Executing bluetooth_pair driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        debug!("Pairing with device: {}", mac_address);
        pair_device(mac_address).map_err(|e| DriverError::execution(format!("Failed to pair with device: {}", e)))?;
        info!("Paired with device: {}", mac_address);
        Ok(format!("Paired with device: {}", mac_address))
    }
}
