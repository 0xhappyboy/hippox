//! Bluetooth battery get skill - read device battery level
//!
//! This driver provides functionality to query the battery level of connected
//! Bluetooth devices that support the Battery Service.
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
/// Driver for getting Bluetooth device battery level
///
/// This driver reads the battery level from a connected Bluetooth device
/// that supports the Battery Service (e.g., headphones, speakers).
#[derive(Debug)]
pub struct BluetoothBatteryGetDriver;
#[async_trait::async_trait]
impl Driver for BluetoothBatteryGetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_battery_get"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the battery level of a connected Bluetooth device (if supported)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to check the battery level of Bluetooth headphones, speakers, or other devices that support the Battery Service."
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
            "action": "bluetooth_battery_get",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Battery level: 75%".to_string()
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
        debug!("Executing bluetooth_battery_get driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        debug!("Querying battery level for: {}", mac_address);
        #[cfg(target_os = "linux")]
        {
            debug!("Getting device info via bluetoothctl");
            let output = Command::new("bluetoothctl")
                .args(["info", mac_address])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            debug!("Device info retrieved, scanning for battery information");
            for line in stdout.lines() {
                if line.contains("Battery") || line.contains("battery") {
                    if let Some(percentage) = line.split_whitespace().find(|w| w.contains("%")) {
                        info!("Battery level found: {}", percentage);
                        return Ok(format!("Battery level: {}", percentage));
                    }
                }
            }
        }
        info!("Battery level not available for {}", mac_address);
        Ok(format!("Battery level not available for {}", mac_address))
    }
}
