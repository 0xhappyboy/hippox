//! Bluetooth BLE read skill - read characteristic values from BLE device
//!
//! This driver provides functionality to read characteristic values from
//! BLE devices such as sensors, fitness trackers, and other peripherals.
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
/// Driver for reading BLE characteristic values
///
/// This driver reads the current value of a specified characteristic from
/// a BLE device, useful for reading sensor data or device state.
#[derive(Debug)]
pub struct BluetoothBleReadDriver;
#[async_trait::async_trait]
impl Driver for BluetoothBleReadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_ble_read"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Read a characteristic value from a BLE device"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to read sensor data or device state from a BLE device. Requires the characteristic UUID."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the BLE device".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "characteristic_uuid".to_string(),
                param_type: "string".to_string(),
                description: "UUID of the characteristic to read".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("00002a19-0000-1000-8000-00805f9b34fb".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_ble_read",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "characteristic_uuid": "00002a19-0000-1000-8000-00805f9b34fb"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Characteristic value: 0x64 (100)".to_string()
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
        debug!("Executing bluetooth_ble_read driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        let characteristic_uuid = parameters.get("characteristic_uuid").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'characteristic_uuid' parameter");
            DriverError::missing_parameter("characteristic_uuid")
        })?;
        debug!("Reading characteristic {} from {}", characteristic_uuid, mac_address);
        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl get-value {} {}", mac_address, characteristic_uuid);
            let output = Command::new("bluetoothctl")
                .args(["get-value", mac_address, characteristic_uuid])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let value = stdout.trim();
            info!("Characteristic value read: {}", value);
            return Ok(format!("Characteristic value: {}", value));
        }
        info!("Read from {} characteristic {}", mac_address, characteristic_uuid);
        Ok(format!("Read from {} characteristic {}", mac_address, characteristic_uuid))
    }
}
