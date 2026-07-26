//! Bluetooth send file skill - send file via OBEX push
//!
//! This driver provides functionality to send a file to a Bluetooth
//! device via OBEX Object Push.
use super::common::send_file;
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
/// Driver for sending files via Bluetooth
///
/// This driver sends a file to a paired Bluetooth device via OBEX
/// Object Push, allowing file transfers to phones and other devices.
#[derive(Debug)]
pub struct BluetoothSendFileDriver;
#[async_trait::async_trait]
impl Driver for BluetoothSendFileDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_send_file"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Send a file to a Bluetooth device via OBEX Object Push"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to send files (photos, documents, etc.) to a paired Bluetooth device like a phone."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the target device".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "file_path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the file to send".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/home/user/photo.jpg".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_send_file",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "file_path": "/home/user/photo.jpg"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "File sent successfully to AA:BB:CC:DD:EE:FF".to_string()
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
        debug!("Executing bluetooth_send_file driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        let file_path = parameters.get("file_path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'file_path' parameter");
            DriverError::missing_parameter("file_path")
        })?;
        debug!("Sending file {} to {}", file_path, mac_address);
        if !Path::new(file_path).exists() {
            debug!("File does not exist: {}", file_path);
            return Err(DriverError::execution(format!("File does not exist: {}", file_path)));
        }
        send_file(mac_address, file_path).map_err(|e| DriverError::execution(format!("Failed to send file: {}", e)))?;
        info!("File sent successfully to {}", mac_address);
        Ok(format!("File sent successfully to {}", mac_address))
    }
}
