//! Bluetooth set device name skill - change the Bluetooth adapter name
//!
//! This driver provides functionality to change the name of the local
//! Bluetooth adapter that other devices see.
use super::common::set_device_name;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for setting the Bluetooth adapter name
///
/// This driver changes the name of the system's Bluetooth adapter,
/// which is how it appears to other Bluetooth devices.
#[derive(Debug)]
pub struct BluetoothSetDeviceNameDriver;
#[async_trait::async_trait]
impl Driver for BluetoothSetDeviceNameDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_set_device_name"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Change the Bluetooth adapter name that other devices see"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to customize how your device appears to other Bluetooth devices."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "name".to_string(),
            param_type: "string".to_string(),
            description: "New Bluetooth device name (max 248 characters)".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("My Computer".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_set_device_name",
            "parameters": {
                "name": "My Computer"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Bluetooth device name set to: My Computer".to_string()
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
        debug!("Executing bluetooth_set_device_name driver");
        let name = parameters.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'name' parameter");
            DriverError::missing_parameter("name")
        })?;
        // Validate name length (max 248 characters as per Bluetooth spec)
        if name.len() > 248 {
            debug!("Device name exceeds 248 characters: {}", name.len());
            return Err(DriverError::validation("name", "Device name must be 248 characters or less"));
        }
        debug!("Setting Bluetooth device name to: {}", name);
        set_device_name(name).map_err(|e| DriverError::execution(format!("Failed to set device name: {}", e)))?;
        info!("Bluetooth device name set to: {}", name);
        Ok(format!("Bluetooth device name set to: {}", name))
    }
}
