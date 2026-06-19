//! Bluetooth trust device skill - mark device as trusted for auto-accept pairing

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
pub struct BluetoothTrustDeviceDriver;

#[async_trait::async_trait]
impl Driver for BluetoothTrustDeviceDriver {
    fn name(&self) -> &str {
        "bluetooth_trust_device"
    }

    fn description(&self) -> &str {
        "Mark a Bluetooth device as trusted (auto-accept future pairing requests)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to trust a device so it can connect automatically without confirmation."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "mac_address".to_string(),
                param_type: "string".to_string(),
                description: "MAC address of the device to trust".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("AA:BB:CC:DD:EE:FF".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "trust".to_string(),
                param_type: "boolean".to_string(),
                description: "True to trust, false to untrust".to_string(),
                required: true,
                default: None,
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_trust_device",
            "parameters": {
                "mac_address": "AA:BB:CC:DD:EE:FF",
                "trust": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Device AA:BB:CC:DD:EE:FF is now trusted".to_string()
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
        let trust = parameters
            .get("trust")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow::anyhow!("Missing 'trust' parameter"))?;
        let action = if trust { "trust" } else { "untrust" };
        #[cfg(target_os = "linux")]
        {
            Command::new("bluetoothctl")
                .args([action, mac_address])
                .output()?;
        }
        if trust {
            Ok(format!("Device {} is now trusted", mac_address))
        } else {
            Ok(format!("Device {} is no longer trusted", mac_address))
        }
    }
}
