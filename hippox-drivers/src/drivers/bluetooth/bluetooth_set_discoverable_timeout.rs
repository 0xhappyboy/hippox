//! Bluetooth set discoverable timeout skill - set how long device stays discoverable
//!
//! This driver provides functionality to set how long the Bluetooth adapter
//! remains discoverable.
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
/// Driver for setting Bluetooth discoverable timeout
///
/// This driver controls how long the Bluetooth adapter remains discoverable.
/// Set to 0 for unlimited discoverability.
#[derive(Debug)]
pub struct BluetoothSetDiscoverableTimeoutDriver;
#[async_trait::async_trait]
impl Driver for BluetoothSetDiscoverableTimeoutDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_set_discoverable_timeout"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set how long the Bluetooth adapter remains discoverable (0 = unlimited)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to control the discoverable duration. Set to 0 for unlimited, or a positive number for limited time."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "timeout_secs".to_string(),
            param_type: "integer".to_string(),
            description: "Discoverable timeout in seconds (0 = unlimited)".to_string(),
            required: true,
            default: None,
            example: Some(Value::Number(60.into())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_set_discoverable_timeout",
            "parameters": {
                "timeout_secs": 60
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Discoverable timeout set to 60 seconds".to_string()
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
        debug!("Executing bluetooth_set_discoverable_timeout driver");
        let timeout = parameters.get("timeout_secs").and_then(|v| v.as_u64()).ok_or_else(|| {
            debug!("Missing 'timeout_secs' parameter");
            DriverError::missing_parameter("timeout_secs")
        })?;
        debug!("Setting discoverable timeout to: {}s", timeout);
        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl discoverable-timeout {}", timeout);
            let output = Command::new("bluetoothctl")
                .args(["discoverable-timeout", &timeout.to_string()])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("bluetoothctl discoverable-timeout failed: {}", stderr);
                return Err(DriverError::execution(format!("Failed to set discoverable timeout: {}", stderr)));
            }
        }
        let result = if timeout == 0 {
            "Discoverable timeout set to unlimited".to_string()
        } else {
            format!("Discoverable timeout set to {} seconds", timeout)
        };
        info!("{}", result);
        Ok(result)
    }
}
