//! Bluetooth scan skill - scan for nearby Bluetooth devices
//!
//! This driver provides functionality to scan for nearby Bluetooth devices
//! and display their information.
use super::common::scan_devices;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for scanning Bluetooth devices
///
/// This driver scans for nearby Bluetooth devices and returns their
/// names, MAC addresses, and types.
#[derive(Debug)]
pub struct BluetoothScanDriver;
#[async_trait::async_trait]
impl Driver for BluetoothScanDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_scan"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Scan for nearby Bluetooth devices and return their names, MAC addresses, and types"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to discover Bluetooth devices in range. Scanning may take 5-10 seconds."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "timeout_secs".to_string(),
            param_type: "integer".to_string(),
            description: "Scan timeout in seconds (default: 10)".to_string(),
            required: false,
            default: Some(Value::Number(10.into())),
            example: Some(Value::Number(15.into())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_scan",
            "parameters": {
                "timeout_secs": 10
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Found 3 devices:\n1. My Headphones (AA:BB:CC:DD:EE:FF) [Audio]\n2. My Phone (11:22:33:44:55:66) [Phone]\n3. Mouse (77:88:99:AA:BB:CC) [HID]"
            .to_string()
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
        debug!("Executing bluetooth_scan driver");
        let timeout = parameters.get("timeout_secs").and_then(|v| v.as_u64()).unwrap_or(10);
        debug!("Scanning for devices with timeout: {}s", timeout);
        // Wait for scan to complete
        tokio::time::sleep(std::time::Duration::from_secs(timeout)).await;
        let devices = scan_devices().map_err(|e| DriverError::execution(format!("Failed to scan devices: {}", e)))?;
        if devices.is_empty() {
            debug!("No Bluetooth devices found");
            return Ok("No Bluetooth devices found".to_string());
        }
        debug!("Found {} devices", devices.len());
        let mut result = format!("Found {} devices:\n", devices.len());
        for (i, device) in devices.iter().enumerate() {
            let paired_marker = if device.paired { " [PAIRED]" } else { "" };
            let connected_marker = if device.connected { " [CONNECTED]" } else { "" };
            result.push_str(&format!("{}. {}{}{} ({})", i + 1, device.name, paired_marker, connected_marker, device.mac_address));
            if let Some(rssi) = device.rssi {
                result.push_str(&format!(" (Signal: {} dBm)", rssi));
            }
            result.push('\n');
        }
        info!("Scan completed, found {} devices", devices.len());
        Ok(result)
    }
}
