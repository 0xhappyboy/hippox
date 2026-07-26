//! Bluetooth BLE write skill - write characteristic values to BLE device
//!
//! This driver provides functionality to write characteristic values to
//! BLE devices for control and configuration purposes.
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
/// Driver for writing BLE characteristic values
///
/// This driver writes a value to a specified characteristic on a BLE device,
/// useful for controlling devices like lights, locks, or sensors.
#[derive(Debug)]
pub struct BluetoothBleWriteDriver;
#[async_trait::async_trait]
impl Driver for BluetoothBleWriteDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_ble_write"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Write a characteristic value to a BLE device"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to control BLE devices like lights, locks, or sensors. Requires the characteristic UUID."
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
                description: "UUID of the characteristic to write".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("00002a19-0000-1000-8000-00805f9b34fb".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "value".to_string(),
                param_type: "string".to_string(),
                description: "Value to write (hex format like '0x01' or string)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("0x01".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_ble_write",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "characteristic_uuid": "00002a19-0000-1000-8000-00805f9b34fb",
                "value": "0x01"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Characteristic value written successfully".to_string()
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
        debug!("Executing bluetooth_ble_write driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        let characteristic_uuid = parameters.get("characteristic_uuid").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'characteristic_uuid' parameter");
            DriverError::missing_parameter("characteristic_uuid")
        })?;
        let value = parameters.get("value").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'value' parameter");
            DriverError::missing_parameter("value")
        })?;
        debug!("Writing value '{}' to characteristic {} on {}", value, characteristic_uuid, mac_address);
        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl set-value {} {} {}", mac_address, characteristic_uuid, value);
            let output = Command::new("bluetoothctl")
                .args(["set-value", mac_address, characteristic_uuid, value])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("bluetoothctl set-value failed: {}", stderr);
                return Err(DriverError::execution(format!("Failed to write value: {}", stderr)));
            }
        }
        info!("Characteristic value written successfully");
        Ok("Characteristic value written successfully".to_string())
    }
}
