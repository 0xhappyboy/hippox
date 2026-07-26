//! Bluetooth RSSI get skill - get signal strength of connected device
//!
//! This driver provides functionality to query the RSSI (signal strength)
//! of a connected Bluetooth device.
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
/// Driver for getting Bluetooth RSSI
///
/// This driver retrieves the signal strength (RSSI) of a connected
/// Bluetooth device. Values closer to 0 indicate better signal quality.
#[derive(Debug)]
pub struct BluetoothRssiGetDriver;
#[async_trait::async_trait]
impl Driver for BluetoothRssiGetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_rssi_get"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the RSSI (signal strength) of a connected Bluetooth device"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to check signal strength. Values closer to 0 are better (e.g., -30 dBm is excellent, -90 dBm is poor)."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the device".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_rssi_get",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "RSSI: -45 dBm (Good signal)".to_string()
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
        debug!("Executing bluetooth_rssi_get driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        debug!("Getting RSSI for: {}", mac_address);
        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl info {}", mac_address);
            let output = Command::new("bluetoothctl")
                .args(["info", mac_address])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            debug!("Device info retrieved, scanning for RSSI");
            for line in stdout.lines() {
                if line.contains("RSSI:") {
                    if let Some(rssi) = line.split(':').nth(1) {
                        let rssi_val: i32 = rssi.trim().parse().unwrap_or(0);
                        let quality = if rssi_val > -50 {
                            "Excellent"
                        } else if rssi_val > -70 {
                            "Good"
                        } else if rssi_val > -85 {
                            "Fair"
                        } else {
                            "Poor"
                        };
                        info!("RSSI: {} dBm ({})", rssi_val, quality);
                        return Ok(format!("RSSI: {} dBm ({})", rssi_val, quality));
                    }
                }
            }
        }
        info!("RSSI not available for {}", mac_address);
        Ok(format!("RSSI not available for {}", mac_address))
    }
}
