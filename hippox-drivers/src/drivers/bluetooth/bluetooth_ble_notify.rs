//! Bluetooth BLE notify skill - subscribe to BLE notifications/indications
//!
//! This driver provides functionality to subscribe or unsubscribe to
//! notifications/indications from BLE device characteristics.
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
/// Driver for BLE notification subscription
///
/// This driver enables or disables notifications/indications from a BLE
/// device characteristic, allowing real-time data updates from sensors
/// and other BLE peripherals.
#[derive(Debug)]
pub struct BluetoothBleNotifyDriver;
#[async_trait::async_trait]
impl Driver for BluetoothBleNotifyDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_ble_notify"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Subscribe to notifications/indications from a BLE device characteristic"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to receive real-time updates from BLE devices like heart rate monitors or sensors."
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
                description: "UUID of the characteristic to subscribe to".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("00002a37-0000-1000-8000-00805f9b34fb".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "enable".to_string(),
                param_type: "boolean".to_string(),
                description: "Enable (true) or disable (false) notifications".to_string(),
                required: true,
                default: None,
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_ble_notify",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "characteristic_uuid": "00002a37-0000-1000-8000-00805f9b34fb",
                "enable": true
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Notifications enabled for characteristic".to_string()
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
        debug!("Executing bluetooth_ble_notify driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        let characteristic_uuid = parameters.get("characteristic_uuid").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'characteristic_uuid' parameter");
            DriverError::missing_parameter("characteristic_uuid")
        })?;
        let enable = parameters.get("enable").and_then(|v| v.as_bool()).ok_or_else(|| {
            debug!("Missing 'enable' parameter");
            DriverError::missing_parameter("enable")
        })?;
        let action = if enable { "notify" } else { "unnotify" };
        debug!("Action: {}, Device: {}, Characteristic: {}", action, mac_address, characteristic_uuid);
        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl {} {} {}", action, mac_address, characteristic_uuid);
            let output = Command::new("bluetoothctl")
                .args([action, mac_address, characteristic_uuid])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("bluetoothctl {} failed: {}", action, stderr);
                return Err(DriverError::execution(format!("Failed to set notifications: {}", stderr)));
            }
        }
        let result =
            if enable { "Notifications enabled for characteristic".to_string() } else { "Notifications disabled for characteristic".to_string() };
        info!("{}", result);
        Ok(result)
    }
}
