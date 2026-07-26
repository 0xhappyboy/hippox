//! Bluetooth auto connect toggle skill - enable/disable auto reconnect
//!
//! This driver provides functionality to enable or disable automatic reconnection
//! for paired Bluetooth devices.
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
/// Driver for toggling Bluetooth auto-connect
///
/// This driver controls whether a paired device will automatically reconnect
/// when it comes back into range or powers on.
#[derive(Debug)]
pub struct BluetoothAutoConnectToggleDriver;
#[async_trait::async_trait]
impl Driver for BluetoothAutoConnectToggleDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_auto_connect_toggle"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Enable or disable auto-reconnect for a paired Bluetooth device"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to control whether a device automatically reconnects when it comes back into range."
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
                name: "enabled".to_string(),
                param_type: "boolean".to_string(),
                description: "Enable (true) or disable (false) auto-connect".to_string(),
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
            "action": "bluetooth_auto_connect_toggle",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "enabled": true
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Auto-connect enabled for AA:BB:CC:DD:EE:FF".to_string()
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
        debug!("Executing bluetooth_auto_connect_toggle driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        let enabled = parameters.get("enabled").and_then(|v| v.as_bool()).ok_or_else(|| {
            debug!("Missing 'enabled' parameter");
            DriverError::missing_parameter("enabled")
        })?;
        debug!("Setting auto-connect for {} to {}", mac_address, enabled);
        #[cfg(target_os = "linux")]
        {
            let action = if enabled { "trust" } else { "untrust" };
            debug!("Executing bluetoothctl {} {}", action, mac_address);
            let output = Command::new("bluetoothctl")
                .args([action, mac_address])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("bluetoothctl {} failed: {}", action, stderr);
                return Err(DriverError::execution(format!("Failed to set auto-connect: {}", stderr)));
            }
        }
        let result = if enabled { format!("Auto-connect enabled for {}", mac_address) } else { format!("Auto-connect disabled for {}", mac_address) };
        info!("{}", result);
        Ok(result)
    }
}
