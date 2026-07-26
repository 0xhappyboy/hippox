//! WiFi scan skill - scan for nearby WiFi networks
use super::common::scan_wifi_networks;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for scanning WiFi networks
#[derive(Debug)]
pub struct WifiScanDriver;
#[async_trait::async_trait]
impl Driver for WifiScanDriver {
    fn name(&self) -> &str {
        return "wifi_scan";
    }
    fn description(&self) -> &str {
        return "Scan for nearby WiFi networks and return SSID, signal strength, and encryption type";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to discover available WiFi networks in the area. Returns a list of networks sorted by signal strength.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "timeout_secs".to_string(),
            param_type: "integer".to_string(),
            description: "Scan timeout in seconds (default: 10)".to_string(),
            required: false,
            default: Some(Value::Number(10.into())),
            example: Some(Value::Number(15.into())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_scan",
            "parameters": {
                "timeout_secs": 10
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Found 5 networks:\n1. MyWiFi (Signal: 85%, Security: WPA2-Personal)\n2. GuestWiFi (Signal: 45%, Security: Open)\n3. OfficeNet (Signal: 30%, Security: WPA2-Enterprise)".to_string();
    }
    fn category(&self) -> DriverCategory {
        return DriverCategory::Wifi;
    }
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing wifi_scan driver");
        let timeout = parameters.get("timeout_secs").and_then(|v| v.as_u64()).unwrap_or(10);
        // Wait for scan to complete
        tokio::time::sleep(std::time::Duration::from_secs(timeout)).await;
        let networks = scan_wifi_networks().map_err(|e| {
            debug!("Failed to scan WiFi networks: {}", e);
            return DriverError::execution(format!("Failed to scan WiFi networks: {}", e));
        })?;
        if networks.is_empty() {
            info!("No WiFi networks found");
            return Ok("No WiFi networks found".to_string());
        }
        let mut result = format!("Found {} networks:\n", networks.len());
        for (i, network) in networks.iter().enumerate() {
            let connected_marker = if network.is_connected { " [CONNECTED]" } else { "" };
            result.push_str(&format!(
                "{}. {}{} (Signal: {}%, Security: {})",
                i + 1,
                network.ssid,
                connected_marker,
                network.signal_strength,
                network.encryption_type
            ));
            if let Some(bssid) = &network.bssid {
                result.push_str(&format!(", BSSID: {}", bssid));
            }
            if let Some(channel) = network.channel {
                result.push_str(&format!(", Channel: {}", channel));
            }
            result.push('\n');
        }
        info!("Found {} WiFi networks", networks.len());
        return Ok(result);
    }
}
