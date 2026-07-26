//! Bluetooth delete device skill - remove device from paired list
//!
//! This driver provides functionality to permanently remove a device from
//! the paired devices list.
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
/// Driver for deleting paired Bluetooth devices
///
/// This driver permanently removes a device from the paired devices list.
/// The device will need to be paired again before it can be used.
#[derive(Debug)]
pub struct BluetoothDeleteDeviceDriver;
#[async_trait::async_trait]
impl Driver for BluetoothDeleteDeviceDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_delete_device"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Remove/delete a device from the paired devices list"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to permanently remove a device from your paired devices list."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the device to delete".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_delete_device",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Device AA:BB:CC:DD:EE:FF deleted".to_string()
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
        debug!("Executing bluetooth_delete_device driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        debug!("Deleting device: {}", mac_address);
        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl remove {}", mac_address);
            let output = Command::new("bluetoothctl")
                .args(["remove", mac_address])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("bluetoothctl remove failed: {}", stderr);
                return Err(DriverError::execution(format!("Failed to delete device: {}", stderr)));
            }
        }
        info!("Device {} deleted", mac_address);
        Ok(format!("Device {} deleted", mac_address))
    }
}
