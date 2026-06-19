//! Bluetooth adapter status skill - get adapter status (powered, discoverable, etc.)

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::DriverCallback;
use crate::DriverContext;
use super::common::get_adapter_status;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct BluetoothAdapterStatusDriver;

#[async_trait::async_trait]
impl Driver for BluetoothAdapterStatusDriver {
    fn name(&self) -> &str {
        "bluetooth_adapter_status"
    }

    fn description(&self) -> &str {
        "Get Bluetooth adapter status including power state, discoverability, and device name"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check if Bluetooth is on, discoverable, and get adapter information."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "verbose".to_string(),
            param_type: "boolean".to_string(),
            description: "Show detailed adapter information".to_string(),
            required: false,
            default: Some(Value::Bool(false)),
            example: Some(Value::Bool(true)),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "bluetooth_adapter_status",
            "parameters": {
                "verbose": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Bluetooth Status:\n- Powered On: Yes\n- Discoverable: Yes\n- Name: My Computer\n- MAC Address: AA:BB:CC:DD:EE:FF".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Bluetooth
    }

    async fn execute(&self, parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,) -> Result<String> {
        let verbose = parameters
            .get("verbose")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let status = get_adapter_status()?;

        let mut result = String::from("Bluetooth Status:\n");
        result.push_str(&format!(
            "- Powered On: {}\n",
            if status.powered_on { "Yes" } else { "No" }
        ));
        result.push_str(&format!(
            "- Discoverable: {}\n",
            if status.discoverable { "Yes" } else { "No" }
        ));
        result.push_str(&format!(
            "- Pairable: {}\n",
            if status.pairable { "Yes" } else { "No" }
        ));

        if verbose {
            result.push_str(&format!("- Name: {}\n", status.name));
            result.push_str(&format!("- MAC Address: {}\n", status.mac_address));
            result.push_str(&format!(
                "- Discoverable Timeout: {} seconds\n",
                status.discoverable_timeout
            ));
        }

        Ok(result)
    }
}
