//! Bluetooth HID connect skill - connect to HID device (keyboard, mouse)
//!
//! This driver provides functionality to connect to Bluetooth HID devices
//! such as keyboards, mice, and gamepads.
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
/// Driver for connecting to Bluetooth HID devices
///
/// This driver establishes a connection to a paired Bluetooth HID device
/// such as a keyboard, mouse, or gamepad.
#[derive(Debug)]
pub struct BluetoothHidConnectDriver;
#[async_trait::async_trait]
impl Driver for BluetoothHidConnectDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_hid_connect"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Connect to a Bluetooth HID device (keyboard, mouse, gamepad)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to connect to Bluetooth input devices. The device must be paired first."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the HID device".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_hid_connect",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Connected to HID device: AA:BB:CC:DD:EE:FF".to_string()
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
        debug!("Executing bluetooth_hid_connect driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        debug!("Connecting to HID device: {}", mac_address);
        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl connect {}", mac_address);
            let output = Command::new("bluetoothctl")
                .args(["connect", mac_address])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("bluetoothctl connect failed: {}", stderr);
                return Err(DriverError::execution(format!("Failed to connect to HID device: {}", stderr)));
            }
        }
        info!("Connected to HID device: {}", mac_address);
        Ok(format!("Connected to HID device: {}", mac_address))
    }
}
