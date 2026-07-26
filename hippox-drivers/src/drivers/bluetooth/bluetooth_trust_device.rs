//! Bluetooth trust device skill - mark device as trusted for auto-accept pairing
//!
//! This driver provides functionality to mark a Bluetooth device as trusted,
//! allowing automatic acceptance of future pairing requests.
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
/// Driver for trusting Bluetooth devices
///
/// This driver marks a device as trusted, which allows future pairing
/// requests to be accepted automatically without user confirmation.
#[derive(Debug)]
pub struct BluetoothTrustDeviceDriver;
#[async_trait::async_trait]
impl Driver for BluetoothTrustDeviceDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_trust_device"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Mark a Bluetooth device as trusted (auto-accept future pairing requests)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to trust a device so it can connect automatically without confirmation."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the device to trust".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "trust".to_string(),
                param_type: "boolean".to_string(),
                description: "True to trust, false to untrust".to_string(),
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
            "action": "bluetooth_trust_device",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "trust": true
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Device AA:BB:CC:DD:EE:FF is now trusted".to_string()
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
        debug!("Executing bluetooth_trust_device driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        let trust = parameters.get("trust").and_then(|v| v.as_bool()).ok_or_else(|| {
            debug!("Missing 'trust' parameter");
            DriverError::missing_parameter("trust")
        })?;
        let action = if trust { "trust" } else { "untrust" };
        debug!("Setting trust for {}: {}", mac_address, action);
        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl {} {}", action, mac_address);
            let output = Command::new("bluetoothctl")
                .args([action, mac_address])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("bluetoothctl {} failed: {}", action, stderr);
                return Err(DriverError::execution(format!("Failed to set trust: {}", stderr)));
            }
        }
        let result = if trust { format!("Device {} is now trusted", mac_address) } else { format!("Device {} is no longer trusted", mac_address) };
        info!("{}", result);
        Ok(result)
    }
}
