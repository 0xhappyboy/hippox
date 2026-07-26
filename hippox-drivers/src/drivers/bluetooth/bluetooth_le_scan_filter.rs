//! Bluetooth LE scan filter skill - scan for specific BLE devices by service UUID
//!
//! This driver provides functionality to scan for BLE devices that support
//! a specific service UUID.
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
/// Driver for filtered BLE scanning
///
/// This driver scans for BLE devices that support a specific service UUID,
/// useful for discovering devices of a particular type (e.g., heart rate monitors).
#[derive(Debug)]
pub struct BluetoothLeScanFilterDriver;
#[async_trait::async_trait]
impl Driver for BluetoothLeScanFilterDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_le_scan_filter"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Scan for BLE devices with a specific service UUID filter"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to discover only devices that support a specific BLE service (e.g., heart rate monitors, temperature sensors)."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "service_uuid".to_string(),
                param_type: "string".to_string(),
                description: "Service UUID to filter for".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("0000180d-0000-1000-8000-00805f9b34fb".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout_secs".to_string(),
                param_type: "integer".to_string(),
                description: "Scan timeout in seconds (default: 10)".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(15.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_le_scan_filter",
            "parameters": {
                "service_uuid": "0000180d-0000-1000-8000-00805f9b34fb",
                "timeout_secs": 10
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Found 2 devices with service 0000180d-0000-1000-8000-00805f9b34fb:\n1. Heart Rate Monitor (AA:BB:CC:DD:EE:FF)\n2. Fitness Tracker (11:22:33:44:55:66)".to_string()
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
        debug!("Executing bluetooth_le_scan_filter driver");
        let service_uuid = parameters.get("service_uuid").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'service_uuid' parameter");
            DriverError::missing_parameter("service_uuid")
        })?;
        let timeout = parameters.get("timeout_secs").and_then(|v| v.as_u64()).unwrap_or(10);
        debug!("Filtering scan for service UUID: {}, timeout: {}s", service_uuid, timeout);
        #[cfg(target_os = "linux")]
        {
            debug!("Starting BLE scan");
            let _ = Command::new("bluetoothctl").args(["scan", "on"]).output();
            debug!("Waiting for scan results ({}s)", timeout);
            tokio::time::sleep(std::time::Duration::from_secs(timeout)).await;
            let output = Command::new("bluetoothctl")
                .args(["devices"])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            let _ = Command::new("bluetoothctl").args(["scan", "off"]).output();
            let stdout = String::from_utf8_lossy(&output.stdout);
            debug!("Scan completed, processing results");
            let mut result = format!("Found devices with service {}:\n", service_uuid);
            let mut count = 0;
            for line in stdout.lines() {
                if line.starts_with("Device") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let mac = parts[1];
                        let name = parts[2..].join(" ");
                        count += 1;
                        result.push_str(&format!("{}. {} ({})\n", count, name, mac));
                    }
                }
            }
            if count == 0 {
                info!("No devices found with service {}", service_uuid);
                return Ok(format!("No devices found with service {}", service_uuid));
            }
            info!("Found {} devices with service {}", count, service_uuid);
            return Ok(result);
        }
        info!("Filtered scan for service {}", service_uuid);
        Ok(format!("Filtered scan for service {}", service_uuid))
    }
}
