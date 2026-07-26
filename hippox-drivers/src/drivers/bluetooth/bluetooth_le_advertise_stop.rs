//! Bluetooth LE advertise stop skill - stop BLE advertising
//!
//! This driver provides functionality to stop BLE advertising.

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

/// Driver for stopping BLE advertising
///
/// This driver stops BLE advertising that was started by
/// bluetooth_le_advertise_start.
#[derive(Debug)]
pub struct BluetoothLeAdvertiseStopDriver;

#[async_trait::async_trait]
impl Driver for BluetoothLeAdvertiseStopDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_le_advertise_stop"
    }

    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Stop BLE advertising"
    }

    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to stop BLE broadcasting started by bluetooth_le_advertise_start."
    }

    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }

    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_le_advertise_stop"
        }));
    }

    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "BLE advertising stopped".to_string()
    }

    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        DriverCategory::Bluetooth
    }

    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing bluetooth_le_advertise_stop driver");

        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl advertise off");
            let output = Command::new("bluetoothctl")
                .args(["advertise", "off"])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("bluetoothctl advertise off failed: {}", stderr);
                return Err(DriverError::execution(format!("Failed to stop advertising: {}", stderr)));
            }
        }

        info!("BLE advertising stopped");
        Ok("BLE advertising stopped".to_string())
    }
}
