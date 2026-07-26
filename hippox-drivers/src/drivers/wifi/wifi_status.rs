//! WiFi status skill - get current WiFi connection status
use super::common::get_wifi_status;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting WiFi status
#[derive(Debug)]
pub struct WifiStatusDriver;
#[async_trait::async_trait]
impl Driver for WifiStatusDriver {
    fn name(&self) -> &str {
        return "wifi_status";
    }
    fn description(&self) -> &str {
        return "Get current WiFi connection status including SSID, signal strength, IP address, and channel";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to check if WiFi is connected and get detailed connection information.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "verbose".to_string(),
            param_type: "boolean".to_string(),
            description: "Show detailed information including BSSID and link speed".to_string(),
            required: false,
            default: Some(Value::Bool(false)),
            example: Some(Value::Bool(true)),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_status",
            "parameters": {
                "verbose": true
            }
        }));
    }
    fn example_output(&self) -> String {
        return "WiFi Status:\n- Connected: Yes\n- SSID: MyWiFi\n- IP Address: 192.168.1.100\n- Signal Strength: 85%\n- Channel: 6 (2.4GHz)"
            .to_string();
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
        debug!("Executing wifi_status driver");
        let verbose = parameters.get("verbose").and_then(|v| v.as_bool()).unwrap_or(false);
        let status = get_wifi_status().map_err(|e| {
            debug!("Failed to get WiFi status: {}", e);
            return DriverError::execution(format!("Failed to get WiFi status: {}", e));
        })?;
        if !status.connected {
            info!("WiFi is not connected");
            return Ok("WiFi is not connected".to_string());
        }
        let mut result = String::from("WiFi Status:\n");
        result.push_str("- Connected: Yes\n");
        if let Some(ssid) = &status.ssid {
            result.push_str(&format!("- SSID: {}\n", ssid));
        }
        if let Some(ip) = &status.ip_address {
            result.push_str(&format!("- IP Address: {}\n", ip));
        }
        if let Some(signal) = status.signal_strength {
            result.push_str(&format!("- Signal Strength: {}%\n", signal));
        }
        if let Some(channel) = status.channel {
            let freq = if channel <= 14 {
                "2.4GHz"
            } else if channel <= 64 {
                "5GHz"
            } else {
                "6GHz"
            };
            result.push_str(&format!("- Channel: {} ({})\n", channel, freq));
        }
        if verbose {
            if let Some(bssid) = &status.bssid {
                result.push_str(&format!("- BSSID: {}\n", bssid));
            }
            if let Some(speed) = status.link_speed {
                result.push_str(&format!("- Link Speed: {} Mbps\n", speed));
            }
        }
        info!("WiFi status retrieved for SSID: {:?}", status.ssid);
        return Ok(result);
    }
}
