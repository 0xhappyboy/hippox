//! Bluetooth set receive directory skill - set directory for received files
//!
//! This driver provides functionality to set the directory where received
//! Bluetooth files will be saved.
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};
/// Driver for setting Bluetooth receive directory
///
/// This driver configures the directory where files received via Bluetooth
/// OBEX Object Push will be saved.
#[derive(Debug)]
pub struct BluetoothSetReceiveDirectoryDriver;
#[async_trait::async_trait]
impl Driver for BluetoothSetReceiveDirectoryDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_set_receive_directory"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set the directory where Bluetooth received files will be saved"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to change where received Bluetooth files are stored."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "directory".to_string(),
            param_type: "string".to_string(),
            description: "Path to the directory for received files".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/home/user/Downloads/Bluetooth".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_set_receive_directory",
            "parameters": {
                "directory": "/home/user/Downloads/Bluetooth"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Receive directory set to: /home/user/Downloads/Bluetooth".to_string()
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
        debug!("Executing bluetooth_set_receive_directory driver");
        let directory = parameters.get("directory").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'directory' parameter");
            DriverError::missing_parameter("directory")
        })?;
        debug!("Setting receive directory to: {}", directory);
        // Create directory if it doesn't exist
        if !Path::new(directory).exists() {
            debug!("Creating directory: {}", directory);
            std::fs::create_dir_all(directory).map_err(|e| DriverError::execution(format!("Failed to create directory: {}", e)))?;
        }
        #[cfg(target_os = "linux")]
        {
            // Configure obexftp directory
            debug!("Configuring obexd.conf");
            std::fs::write("/etc/bluetooth/obexd.conf", format!("[General]\nDirectory={}\n", directory))
                .map_err(|e| DriverError::execution(format!("Failed to write obexd.conf: {}", e)))?;
        }
        info!("Receive directory set to: {}", directory);
        Ok(format!("Receive directory set to: {}", directory))
    }
}
