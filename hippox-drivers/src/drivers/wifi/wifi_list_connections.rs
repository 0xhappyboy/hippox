//! WiFi list connections skill - list saved/connected WiFi networks

use super::common::{get_wifi_status, list_saved_networks};
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct WifiListConnectionsDriver;

#[async_trait::async_trait]
impl Driver for WifiListConnectionsDriver {
    fn name(&self) -> &str {
        "wifi_list_connections"
    }

    fn description(&self) -> &str {
        "List all saved WiFi networks and show which one is currently connected"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see what WiFi networks have been saved on this device and which one is currently active."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_list_connections"
        })
    }

    fn example_output(&self) -> String {
        "Saved networks (3):\n1. MyWiFi [Connected]\n2. GuestWiFi\n3. OfficeNet".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Wifi
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let status = get_wifi_status()?;
        let current_ssid = status.ssid.clone();

        let saved_networks = list_saved_networks()?;

        if saved_networks.is_empty() {
            return Ok("No saved WiFi networks found".to_string());
        }

        let mut result = format!("Saved networks ({}):\n", saved_networks.len());
        for (i, network) in saved_networks.iter().enumerate() {
            let connected_marker = if Some(&network.ssid) == current_ssid.as_ref() {
                " [CONNECTED]"
            } else {
                ""
            };
            result.push_str(&format!(
                "{}. {}{}\n",
                i + 1,
                network.ssid,
                connected_marker
            ));
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

        Ok(result)
    }
}
