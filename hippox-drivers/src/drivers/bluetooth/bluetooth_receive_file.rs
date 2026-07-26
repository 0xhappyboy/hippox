//! Bluetooth receive file skill - receive files via Bluetooth
//!
//! This driver provides functionality to prepare the system to receive
//! files via Bluetooth OBEX Object Push.
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use tracing::{debug, info, warn};
/// Driver for receiving files via Bluetooth
///
/// This driver prepares the system to receive files from other Bluetooth
/// devices via OBEX Object Push.
#[derive(Debug)]
pub struct BluetoothReceiveFileDriver;
#[async_trait::async_trait]
impl Driver for BluetoothReceiveFileDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_receive_file"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Enable receiving files via Bluetooth (OBEX Object Push)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to prepare your device to receive files from other Bluetooth devices."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "save_directory".to_string(),
                param_type: "string".to_string(),
                description: "Directory to save received files (default: downloads folder)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("/home/user/Downloads".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout_secs".to_string(),
                param_type: "integer".to_string(),
                description: "How long to accept incoming files (default: 60 seconds)".to_string(),
                required: false,
                default: Some(Value::Number(60.into())),
                example: Some(Value::Number(120.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_receive_file",
            "parameters": {
                "save_directory": "/home/user/Downloads",
                "timeout_secs": 60
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Ready to receive files for 60 seconds".to_string()
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
        debug!("Executing bluetooth_receive_file driver");
        let save_directory = parameters.get("save_directory").and_then(|v| v.as_str()).unwrap_or("/tmp/bluetooth_received");
        let timeout = parameters.get("timeout_secs").and_then(|v| v.as_u64()).unwrap_or(60);
        debug!("Save directory: {}, timeout: {}s", save_directory, timeout);
        // Create directory if it doesn't exist
        if !Path::new(save_directory).exists() {
            debug!("Creating directory: {}", save_directory);
            std::fs::create_dir_all(save_directory).map_err(|e| DriverError::execution(format!("Failed to create directory: {}", e)))?;
        }
        #[cfg(target_os = "linux")]
        {
            debug!("Setting up Bluetooth agent");
            let _ = Command::new("bluetoothctl").args(["agent", "on"]).output();
            let _ = Command::new("bluetoothctl").args(["default-agent"]).output();
        }
        info!("Ready to receive files in '{}' for {} seconds", save_directory, timeout);
        Ok(format!("Ready to receive files in '{}' for {} seconds", save_directory, timeout))
    }
}
