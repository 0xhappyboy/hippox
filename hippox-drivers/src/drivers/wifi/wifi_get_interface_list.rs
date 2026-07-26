//! WiFi get interface list skill - list wireless network interfaces
use super::common::list_interfaces;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing WiFi interfaces
#[derive(Debug)]
pub struct WifiGetInterfaceListDriver;
#[async_trait::async_trait]
impl Driver for WifiGetInterfaceListDriver {
    fn name(&self) -> &str {
        return "wifi_get_interface_list";
    }
    fn description(&self) -> &str {
        return "List all wireless network interfaces with their MAC addresses and status";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to identify available WiFi adapters on the system.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_get_interface_list"
        }));
    }
    fn example_output(&self) -> String {
        return "Found 1 interface:\n1. wlan0 (MAC: 00:11:22:33:44:55, State: connected)".to_string();
    }
    fn category(&self) -> DriverCategory {
        return DriverCategory::Wifi;
    }
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing wifi_get_interface_list driver");
        let interfaces = list_interfaces().map_err(|e| {
            debug!("Failed to list interfaces: {}", e);
            return DriverError::execution(format!("Failed to list interfaces: {}", e));
        })?;
        if interfaces.is_empty() {
            info!("No wireless interfaces found");
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
        info!("Listed {} interfaces", interfaces.len());
        return Ok(result);
    }
}
