//! Bluetooth confirm PIN skill - confirm PIN code for pairing
//!
//! This driver provides functionality to confirm a PIN code during the
//! Bluetooth pairing process.
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
/// Driver for confirming Bluetooth PIN codes
///
/// This driver confirms a PIN code during the pairing process, allowing
/// the user to complete pairing with a device that displays a PIN.
#[derive(Debug)]
pub struct BluetoothConfirmPinDriver;
#[async_trait::async_trait]
impl Driver for BluetoothConfirmPinDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_confirm_pin"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Confirm a PIN code to complete Bluetooth pairing"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to respond to a pairing PIN request when a device shows a PIN code."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "pin".to_string(),
            param_type: "string".to_string(),
            description: "PIN code to confirm (usually 4-6 digits)".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("0000".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_confirm_pin",
            "parameters": {
                "pin": "123456"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "PIN code 123456 confirmed".to_string()
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
        debug!("Executing bluetooth_confirm_pin driver");
        let pin = parameters.get("pin").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'pin' parameter");
            DriverError::missing_parameter("pin")
        })?;
        debug!("Confirming PIN: {}", pin);
        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl pin {}", pin);
            let output = Command::new("bluetoothctl")
                .args(["pin", pin])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("bluetoothctl pin failed: {}", stderr);
                return Err(DriverError::execution(format!("Failed to confirm PIN: {}", stderr)));
            }
        }
        info!("PIN code {} confirmed", pin);
        Ok(format!("PIN code {} confirmed", pin))
    }
}
