//! WiFi get interface list skill - list wireless network interfaces

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::list_interfaces;
use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};

#[derive(Debug)]
pub struct WifiGetInterfaceListDriver;

#[async_trait::async_trait]
impl Driver for WifiGetInterfaceListDriver {
    fn name(&self) -> &str {
        "wifi_get_interface_list"
    }

    fn description(&self) -> &str {
        "List all wireless network interfaces with their MAC addresses and status"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to identify available WiFi adapters on the system."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_get_interface_list"
        })
    }

    fn example_output(&self) -> String {
        "Found 1 interface:\n1. wlan0 (MAC: 00:11:22:33:44:55, State: connected)".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Wifi
    }

    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let interfaces = list_interfaces()?;

        if interfaces.is_empty() {
            return Ok("No wireless interfaces found".to_string());
        }

        let mut result = format!("Found {} interface(s):\n", interfaces.len());
        for (i, iface) in interfaces.iter().enumerate() {
            result.push_str(&format!(
                "{}. {} (MAC: {}, State: {}){}\n",
                i + 1,
                iface.name,
                iface.mac_address,
                iface.state,
                if iface.is_default { " [DEFAULT]" } else { "" }
            ));
        }

        Ok(result)
    }
}
