//! Bluetooth audio disconnect skill - disconnect A2DP audio device
//!
//! This driver provides functionality to disconnect from Bluetooth audio devices
//! such as headphones and speakers.
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
/// Driver for disconnecting Bluetooth audio devices
///
/// This driver disconnects an active connection to a Bluetooth audio device
/// while keeping the device paired for future connections.
#[derive(Debug)]
pub struct BluetoothAudioDisconnectDriver;
#[async_trait::async_trait]
impl Driver for BluetoothAudioDisconnectDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_audio_disconnect"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Disconnect a Bluetooth audio device (headphones, speakers)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to disconnect Bluetooth headphones or speakers."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the audio device".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_audio_disconnect",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Disconnected from audio device: AA:BB:CC:DD:EE:FF".to_string()
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
        debug!("Executing bluetooth_audio_disconnect driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        debug!("Attempting to disconnect audio device: {}", mac_address);
        #[cfg(target_os = "linux")]
        {
            debug!("Disconnecting via bluetoothctl");
            let output = Command::new("bluetoothctl")
                .args(["disconnect", mac_address])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("bluetoothctl disconnect failed: {}", stderr);
                return Err(DriverError::execution(format!("Failed to disconnect: {}", stderr)));
            }
        }
        info!("Disconnected from audio device: {}", mac_address);
        Ok(format!("Disconnected from audio device: {}", mac_address))
    }
}
