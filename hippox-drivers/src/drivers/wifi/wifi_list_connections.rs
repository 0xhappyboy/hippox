// wifi_list_connections.rs
//! WiFi list connections skill - list saved/connected WiFi networks
use super::common::{get_wifi_status, list_saved_networks};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing saved WiFi connections
#[derive(Debug)]
pub struct WifiListConnectionsDriver;
#[async_trait::async_trait]
impl Driver for WifiListConnectionsDriver {
    fn name(&self) -> &str {
        return "wifi_list_connections";
    }
    fn description(&self) -> &str {
        return "List all saved WiFi networks and show which one is currently connected";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see what WiFi networks have been saved on this device and which one is currently active.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_list_connections"
        }));
    }
    fn example_output(&self) -> String {
        return "Saved networks (3):\n1. MyWiFi [Connected]\n2. GuestWiFi\n3. OfficeNet".to_string();
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
        debug!("Executing wifi_list_connections driver");
        let status = get_wifi_status().map_err(|e| {
            debug!("Failed to get WiFi status: {}", e);
            return DriverError::execution(format!("Failed to get WiFi status: {}", e));
        })?;
        let current_ssid = status.ssid.clone();
        let saved_networks = list_saved_networks().map_err(|e| {
            debug!("Failed to list saved networks: {}", e);
            return DriverError::execution(format!("Failed to list saved networks: {}", e));
        })?;
        if saved_networks.is_empty() {
            info!("No saved WiFi networks found");
            return Ok("No saved WiFi networks found".to_string());
        }
        let mut result = format!("Saved networks ({}):\n", saved_networks.len());
        for (i, network) in saved_networks.iter().enumerate() {
            let connected_marker = if Some(&network.ssid) == current_ssid.as_ref() { " [CONNECTED]" } else { "" };
            result.push_str(&format!("{}. {}{}\n", i + 1, network.ssid, connected_marker));
        }
        if let Some(ssid) = current_ssid {
            result.push_str(&format!("\nCurrently connected to: {}", ssid));
            if let Some(ip) = status.ip_address {
                result.push_str(&format!(" (IP: {})", ip));
            }
            if let Some(signal) = status.signal_strength {
                result.push_str(&format!(" (Signal: {}%)", signal));
            }
        } else {
            result.push_str("\nNot currently connected to any WiFi network");
        }
        info!("Listed {} saved networks", saved_networks.len());
        return Ok(result);
    }
}
