//! Bluetooth rename device skill - change display name of a paired device
//!
//! This driver provides functionality to change the display name of a
//! paired Bluetooth device.
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info, warn};
/// Driver for renaming Bluetooth devices
///
/// This driver changes the display name of a paired Bluetooth device,
/// making it easier to identify.
#[derive(Debug)]
pub struct BluetoothRenameDeviceDriver;
#[async_trait::async_trait]
impl Driver for BluetoothRenameDeviceDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_rename_device"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Change the display name of a paired Bluetooth device"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to give a custom name to your Bluetooth devices for easier identification."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the device".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "New name for the device".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("My Headphones".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_rename_device",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "name": "My Headphones"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Device AA:BB:CC:DD:EE:FF renamed to 'My Headphones'".to_string()
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
        debug!("Executing bluetooth_rename_device driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        let name = parameters.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'name' parameter");
            DriverError::missing_parameter("name")
        })?;
        debug!("Renaming device {} to '{}'", mac_address, name);
        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl set-alias {} {}", mac_address, name);
            let output = Command::new("bluetoothctl")
                .args(["set-alias", mac_address, name])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("bluetoothctl set-alias failed: {}", stderr);
                return Err(DriverError::execution(format!("Failed to rename device: {}", stderr)));
            }
        }
        info!("Device {} renamed to '{}'", mac_address, name);
        Ok(format!("Device {} renamed to '{}'", mac_address, name))
    }
}
