//! Bluetooth device info skill - get detailed device information
//!
//! This driver provides detailed information about a Bluetooth device
//! including vendor, RSSI, supported services, and connection status.
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
/// Driver for getting detailed Bluetooth device information
///
/// This driver retrieves comprehensive information about a Bluetooth device
/// including signal strength, manufacturer details, and supported services.
#[derive(Debug)]
pub struct BluetoothDeviceInfoDriver;
#[async_trait::async_trait]
impl Driver for BluetoothDeviceInfoDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_device_info"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get detailed information about a Bluetooth device (vendor, RSSI, supported services)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get detailed information like signal strength and manufacturer info about a device."
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
            "action": "bluetooth_device_info",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Device Info:\n- Name: My Headphones\n- MAC: AA:BB:CC:DD:EE:FF\n- RSSI: -45 dBm\n- Type: Audio\n- Paired: Yes\n- Connected: Yes".to_string()
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
        debug!("Executing bluetooth_device_info driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        debug!("Getting device info for: {}", mac_address);
        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl info {}", mac_address);
            let output = Command::new("bluetoothctl")
                .args(["info", mac_address])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            debug!("Device info retrieved successfully");
            let mut result = format!("Device Info for {}:\n", mac_address);
            for line in stdout.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    result.push_str(&format!("- {}\n", line));
                }
            }
            info!("Device info retrieved for {}", mac_address);
            return Ok(result);
        }
        info!("Device info for {}", mac_address);
        Ok(format!("Device info for {}", mac_address))
    }
}
