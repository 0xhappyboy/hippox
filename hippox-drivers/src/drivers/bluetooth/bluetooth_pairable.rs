//! Bluetooth pairable skill - set device to discoverable/pairable mode
//!
//! This driver provides functionality to make the device discoverable
//! and pairable to other Bluetooth devices.
use super::common::set_discoverable;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for setting discoverable/pairable mode
///
/// This driver makes the system's Bluetooth adapter discoverable and
/// pairable to other Bluetooth devices.
#[derive(Debug)]
pub struct BluetoothPairableDriver;
#[async_trait::async_trait]
impl Driver for BluetoothPairableDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_pairable"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set the device to be discoverable/pairable mode so other devices can find it"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to make your device visible to other Bluetooth devices for pairing."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "enabled".to_string(),
                param_type: "boolean".to_string(),
                description: "Enable (true) or disable (false) discoverable mode".to_string(),
                required: true,
                default: None,
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout_secs".to_string(),
                param_type: "integer".to_string(),
                description: "How long to stay discoverable (default: 120 seconds)".to_string(),
                required: false,
                default: Some(Value::Number(120.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_pairable",
            "parameters": {
                "enabled": true,
                "timeout_secs": 120
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Bluetooth discoverable mode enabled for 120 seconds".to_string()
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
        debug!("Executing bluetooth_pairable driver");
        let enabled = parameters.get("enabled").and_then(|v| v.as_bool()).ok_or_else(|| {
            debug!("Missing 'enabled' parameter");
            DriverError::missing_parameter("enabled")
        })?;
        let timeout = parameters.get("timeout_secs").and_then(|v| v.as_u64()).map(|t| t as u32);
        debug!("Setting discoverable mode: enabled={}, timeout={:?}", enabled, timeout);
        set_discoverable(enabled, timeout).map_err(|e| DriverError::execution(format!("Failed to set discoverable mode: {}", e)))?;
        let result = if enabled {
            if let Some(t) = timeout {
                format!("Bluetooth discoverable mode enabled for {} seconds", t)
            } else {
                "Bluetooth discoverable mode enabled".to_string()
            }
        } else {
            "Bluetooth discoverable mode disabled".to_string()
        };
        info!("{}", result);
        Ok(result)
    }
}
