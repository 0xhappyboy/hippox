//! Bluetooth firmware version skill - get device firmware version
//!
//! This driver provides functionality to query the firmware version of
//! a Bluetooth device if available.
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
/// Driver for getting Bluetooth device firmware version
///
/// This driver attempts to retrieve the firmware version from a Bluetooth
/// device if the device exposes this information.
#[derive(Debug)]
pub struct BluetoothFirmwareVersionDriver;
#[async_trait::async_trait]
impl Driver for BluetoothFirmwareVersionDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_firmware_version"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the firmware version of a Bluetooth device (if available)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to check the firmware version of your Bluetooth device."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the device".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_firmware_version",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Firmware version: 1.2.3".to_string()
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
        debug!("Executing bluetooth_firmware_version driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        debug!("Querying firmware version for: {}", mac_address);
        #[cfg(target_os = "linux")]
        {
            debug!("Getting device info via bluetoothctl");
            let output = Command::new("bluetoothctl")
                .args(["info", mac_address])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            debug!("Device info retrieved, scanning for firmware version");
            for line in stdout.lines() {
                if line.contains("Firmware") || line.contains("Version") {
                    if let Some(ver) = line.split(':').nth(1) {
                        let version = ver.trim().to_string();
                        info!("Firmware version found: {}", version);
                        return Ok(format!("Firmware version: {}", version));
                    }
                }
            }
        }
        info!("Firmware version not available for {}", mac_address);
        Ok(format!("Firmware version not available for {}", mac_address))
    }
}
