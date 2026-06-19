//! Bluetooth RSSI get skill - get signal strength of connected device

use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug)]
pub struct BluetoothRssiGetDriver;

#[async_trait::async_trait]
impl Driver for BluetoothRssiGetDriver {
    fn name(&self) -> &str {
        "bluetooth_rssi_get"
    }

    fn description(&self) -> &str {
        "Get the RSSI (signal strength) of a connected Bluetooth device"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check signal strength. Values closer to 0 are better (e.g., -30 dBm is excellent, -90 dBm is poor)."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "mac_address".to_string(),
            param_type: "string".to_string(),
            description: "MAC address of the device".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_rssi_get",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF"
            }
        })
    }

    fn example_output(&self) -> String {
        "RSSI: -45 dBm (Good signal)".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Bluetooth
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let mac_address = parameters
            .get("mac_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'mac_address' parameter"))?;

        #[cfg(target_os = "linux")]
        {
            let output = Command::new("bluetoothctl")
                .args(["info", mac_address])
                .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
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
                        return Ok(format!("RSSI: {} dBm ({})", rssi_val, quality));
                    }
                }
            }
        }

        Ok(format!("RSSI not available for {}", mac_address))
    }
}
