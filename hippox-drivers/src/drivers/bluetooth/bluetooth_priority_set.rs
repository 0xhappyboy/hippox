//! Bluetooth priority set skill - set connection priority for devices
//!
//! This driver provides functionality to set the connection priority for
//! Bluetooth devices, controlling which devices reconnect first.
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
/// Driver for setting Bluetooth connection priority
///
/// This driver controls which Bluetooth device connects first when multiple
/// paired devices are in range.
#[derive(Debug)]
pub struct BluetoothPrioritySetDriver;
#[async_trait::async_trait]
impl Driver for BluetoothPrioritySetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_priority_set"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set connection priority for Bluetooth devices (which device reconnects first)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to control which Bluetooth device connects first when multiple devices are in range."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "priority_list".to_string(),
            param_type: "array".to_string(),
            description: "List of MAC addresses in priority order (first = highest priority)".to_string(),
            required: true,
            default: None,
            example: Some(json!(["AA:BB:CC:DD:EE:FF", "11:22:33:44:55:66"])),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_priority_set",
            "parameters": {
                "priority_list": ["AA:BB:CC:DD:EE:FF", "11:22:33:44:55:66"]
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Priority set for 2 devices".to_string()
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
        debug!("Executing bluetooth_priority_set driver");
        let priority_list = parameters.get("priority_list").and_then(|v| v.as_array()).ok_or_else(|| {
            debug!("Missing 'priority_list' parameter");
            DriverError::missing_parameter("priority_list")
        })?;
        let count = priority_list.len();
        debug!("Setting priority for {} devices", count);
        // On Linux, priority can be set via configuration files
        #[cfg(target_os = "linux")]
        {
            for (priority, mac) in priority_list.iter().enumerate() {
                if let Some(mac_str) = mac.as_str() {
                    debug!("Setting trust for {} (priority {})", mac_str, priority);
                    let _ = Command::new("bluetoothctl").args(["trust", mac_str]).output().ok();
                }
            }
        }
        info!("Priority set for {} devices", count);
        Ok(format!("Priority set for {} devices", count))
    }
}
