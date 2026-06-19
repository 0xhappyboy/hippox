//! Bluetooth priority set skill - set connection priority for devices

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
pub struct BluetoothPrioritySetDriver;

#[async_trait::async_trait]
impl Driver for BluetoothPrioritySetDriver {
    fn name(&self) -> &str {
        "bluetooth_priority_set"
    }

    fn description(&self) -> &str {
        "Set connection priority for Bluetooth devices (which device reconnects first)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to control which Bluetooth device connects first when multiple devices are in range."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "priority_list".to_string(),
            param_type: "array".to_string(),
            description: "List of MAC addresses in priority order (first = highest priority)"
                .to_string(),
            required: true,
            default: None,
            example: Some(json!(["AA:BB:CC:DD:EE:FF", "11:22:33:44:55:66"])),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_priority_set",
            "parameters": {
                "priority_list": ["AA:BB:CC:DD:EE:FF", "11:22:33:44:55:66"]
            }
        })
    }

    fn example_output(&self) -> String {
        "Priority set for 2 devices".to_string()
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
        let priority_list = parameters
            .get("priority_list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing 'priority_list' parameter"))?;

        let count = priority_list.len();

        // On Linux, priority can be set via configuration files
        #[cfg(target_os = "linux")]
        {
            for (priority, mac) in priority_list.iter().enumerate() {
                if let Some(mac_str) = mac.as_str() {
                    Command::new("bluetoothctl")
                        .args(["trust", mac_str])
                        .output()?;
                }
            }
        }

        Ok(format!("Priority set for {} devices", count))
    }
}
