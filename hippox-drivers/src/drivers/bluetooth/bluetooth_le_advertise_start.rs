//! Bluetooth LE advertise start skill - start BLE advertising
//!
//! This driver provides functionality to start BLE advertising, making
//! the device discoverable to BLE scanners.
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
/// Driver for starting BLE advertising
///
/// This driver starts BLE advertising to broadcast the device's presence
/// to BLE scanners, useful for IoT and sensor applications.
#[derive(Debug)]
pub struct BluetoothLeAdvertiseStartDriver;
#[async_trait::async_trait]
impl Driver for BluetoothLeAdvertiseStartDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_le_advertise_start"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Start BLE advertising to make the device discoverable to BLE scanners"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to broadcast your device's presence to BLE devices. Useful for IoT and sensor applications."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "service_uuid".to_string(),
                param_type: "string".to_string(),
                description: "UUID of the service to advertise (optional)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0000180f-0000-1000-8000-00805f9b34fb".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "manufacturer_data".to_string(),
                param_type: "string".to_string(),
                description: "Manufacturer specific data (hex format)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("0x01020304".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_le_advertise_start",
            "parameters": {
                "service_uuid": "0000180f-0000-1000-8000-00805f9b34fb"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "BLE advertising started".to_string()
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
        debug!("Executing bluetooth_le_advertise_start driver");
        let service_uuid = parameters.get("service_uuid").and_then(|v| v.as_str());
        debug!("Starting BLE advertising, service_uuid: {:?}", service_uuid);
        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl advertise on");
            let output = Command::new("bluetoothctl")
                .args(["advertise", "on"])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("bluetoothctl advertise on failed: {}", stderr);
                return Err(DriverError::execution(format!("Failed to start advertising: {}", stderr)));
            }
            if let Some(uuid) = service_uuid {
                debug!("Setting advertise service: {}", uuid);
                let _ = Command::new("bluetoothctl").args(["advertise", "service", uuid]).output().ok();
            }
        }
        info!("BLE advertising started");
        return Ok("BLE advertising started".to_string());
    }
}
