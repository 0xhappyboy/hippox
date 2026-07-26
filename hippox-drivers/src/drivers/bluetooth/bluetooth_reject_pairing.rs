//! Bluetooth reject pairing skill - reject incoming pairing request
//!
//! This driver provides functionality to reject an incoming Bluetooth
//! pairing request from another device.
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
/// Driver for rejecting Bluetooth pairing requests
///
/// This driver rejects an incoming pairing request from a device,
/// denying the pairing attempt.
#[derive(Debug)]
pub struct BluetoothRejectPairingDriver;
#[async_trait::async_trait]
impl Driver for BluetoothRejectPairingDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_reject_pairing"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Reject an incoming Bluetooth pairing request from another device"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to deny a pairing request from a device."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the device requesting pairing".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_reject_pairing",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Pairing request rejected for AA:BB:CC:DD:EE:FF".to_string()
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
        debug!("Executing bluetooth_reject_pairing driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        debug!("Rejecting pairing request from: {}", mac_address);
        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl reject {}", mac_address);
            let output = Command::new("bluetoothctl")
                .args(["reject", mac_address])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("bluetoothctl reject failed: {}", stderr);
                return Err(DriverError::execution(format!("Failed to reject pairing: {}", stderr)));
            }
        }
        info!("Pairing request rejected for {}", mac_address);
        Ok(format!("Pairing request rejected for {}", mac_address))
    }
}
