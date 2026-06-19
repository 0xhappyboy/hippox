//! Bluetooth set receive directory skill - set directory for received files

use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct BluetoothSetReceiveDirectoryDriver;

#[async_trait::async_trait]
impl Driver for BluetoothSetReceiveDirectoryDriver {
    fn name(&self) -> &str {
        "bluetooth_set_receive_directory"
    }

    fn description(&self) -> &str {
        "Set the directory where Bluetooth received files will be saved"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to change where received Bluetooth files are stored."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "directory".to_string(),
            param_type: "string".to_string(),
            description: "Path to the directory for received files".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/home/user/Downloads/Bluetooth".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_set_receive_directory",
            "parameters": {
                "directory": "/home/user/Downloads/Bluetooth"
            }
        })
    }

    fn example_output(&self) -> String {
        "Receive directory set to: /home/user/Downloads/Bluetooth".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Bluetooth
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let directory = parameters
            .get("directory")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'directory' parameter"))?;

        // Create directory if it doesn't exist
        if !Path::new(directory).exists() {
            std::fs::create_dir_all(directory)?;
        }

        #[cfg(target_os = "linux")]
        {
            // Configure obexftp directory
            std::fs::write(
                "/etc/bluetooth/obexd.conf",
                format!("[General]\nDirectory={}\n", directory),
            )?;
        }

        Ok(format!("Receive directory set to: {}", directory))
    }
}
