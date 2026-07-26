//! Bluetooth scan services skill - scan device services and characteristics (BLE GATT)
//!
//! This driver provides functionality to scan and list available services
//! and characteristics on a BLE device.
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
/// Driver for scanning BLE services
///
/// This driver discovers and lists the available services and
/// characteristics on a BLE device, useful for IoT and sensor devices.
#[derive(Debug)]
pub struct BluetoothScanServicesDriver;
#[async_trait::async_trait]
impl Driver for BluetoothScanServicesDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "bluetooth_scan_services"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Scan and list available services and characteristics on a Bluetooth device (BLE GATT)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to discover what services a BLE device offers, useful for IoT and sensor devices."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the Bluetooth device".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "bluetooth_scan_services",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Found 3 services:\n1. 00001800-0000-1000-8000-00805f9b34fb (Device Information)\n2. 0000180f-0000-1000-8000-00805f9b34fb (Battery Service)\n3. 0000180a-0000-1000-8000-00805f9b34fb (Device Name)".to_string()
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
        debug!("Executing bluetooth_scan_services driver");
        let mac_address = parameters.get("mac_address").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'mac_address' parameter");
            DriverError::missing_parameter("mac_address")
        })?;
        debug!("Scanning services for: {}", mac_address);
        #[cfg(target_os = "linux")]
        {
            debug!("Executing bluetoothctl services {}", mac_address);
            let output = Command::new("bluetoothctl")
                .args(["services", mac_address])
                .output()
                .map_err(|e| DriverError::execution(format!("Failed to execute bluetoothctl: {}", e)))?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            debug!("Service scan completed");
            let mut result = String::new();
            let mut service_count = 0;
            for line in stdout.lines() {
                if line.contains("Service") || line.contains("UUID") {
                    service_count += 1;
                    result.push_str(&format!("{}. {}\n", service_count, line.trim()));
                }
            }
            if service_count == 0 {
                info!("No services found for device {}", mac_address);
                return Ok(format!("No services found for device {}", mac_address));
            }
            info!("Found {} services for {}", service_count, mac_address);
            return Ok(format!("Found {} services:\n{}", service_count, result));
        }
        info!("Service scan for {}", mac_address);
        Ok(format!("Service scan for {}", mac_address))
    }
}
