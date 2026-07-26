//! Bluetooth adapter status skill - get adapter status (powered, discoverable, etc.)
//!
//! This driver provides functionality to query the current status of the Bluetooth
//! adapter, including power state, discoverability, pairability, and device information.
use super::common::get_adapter_status;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for retrieving Bluetooth adapter status
///
/// This driver queries the system's Bluetooth adapter and returns information
/// such as power state, discoverability, pairability, device name, and MAC address.
#[derive(Debug)]
pub struct BluetoothAdapterStatusDriver;
#[async_trait::async_trait]
impl Driver for BluetoothAdapterStatusDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_adapter_status"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get Bluetooth adapter status including power state, discoverability, and device name"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to check if Bluetooth is on, discoverable, and get adapter information."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "verbose".to_string(),
            param_type: "boolean".to_string(),
            description: "Show detailed adapter information".to_string(),
            required: false,
            default: Some(Value::Bool(false)),
            example: Some(Value::Bool(true)),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_adapter_status",
            "parameters": {
                "verbose": true
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Bluetooth Status:\n- Powered On: Yes\n- Discoverable: Yes\n- Name: My Computer\n- MAC Address: AA:BB:CC:DD:EE:FF".to_string()
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
        debug!("Executing bluetooth_adapter_status driver");
        let verbose = parameters.get("verbose").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Verbose mode: {}", verbose);
        let status = get_adapter_status().map_err(|e| DriverError::execution(format!("Failed to get adapter status: {}", e)))?;
        debug!("Adapter status retrieved: powered_on={}, discoverable={}, pairable={}", status.powered_on, status.discoverable, status.pairable);
        let mut result = String::from("Bluetooth Status:\n");
        result.push_str(&format!("- Powered On: {}\n", if status.powered_on { "Yes" } else { "No" }));
        result.push_str(&format!("- Discoverable: {}\n", if status.discoverable { "Yes" } else { "No" }));
        result.push_str(&format!("- Pairable: {}\n", if status.pairable { "Yes" } else { "No" }));
        if verbose {
            debug!("Including verbose adapter information");
            result.push_str(&format!("- Name: {}\n", status.name));
            result.push_str(&format!("- MAC Address: {}\n", status.mac_address));
            result.push_str(&format!("- Discoverable Timeout: {} seconds\n", status.discoverable_timeout));
        }
        info!("Bluetooth adapter status retrieved successfully");
        Ok(result)
    }
}
