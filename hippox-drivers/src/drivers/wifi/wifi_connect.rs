//! WiFi connect skill - connect to a WiFi network
use super::common::connect_wifi;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for connecting to a WiFi network
#[derive(Debug)]
pub struct WifiConnectDriver;
#[async_trait::async_trait]
impl Driver for WifiConnectDriver {
    fn name(&self) -> &str {
        return "wifi_connect";
    }
    fn description(&self) -> &str {
        return "Connect to a WiFi network using SSID and password";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to connect to a WiFi network. Provide the network SSID and password. If the network is open (no password), omit the password parameter.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "ssid".to_string(),
                param_type: "string".to_string(),
                description: "WiFi network name (SSID)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("MyWiFi".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "WiFi password (omit for open networks)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("password123".to_string())),
                enum_values: None,
            },
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_connect",
            "parameters": {
                "ssid": "MyWiFi",
                "password": "password123"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Connected to WiFi network: MyWiFi".to_string();
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
        debug!("Executing wifi_connect driver");
        let ssid = parameters.get("ssid").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'ssid' parameter");
            return DriverError::missing_parameter("ssid");
        })?;
        let password = parameters.get("password").and_then(|v| v.as_str());
        debug!("Connecting to WiFi network: {}", ssid);
        connect_wifi(ssid, password).map_err(|e| {
            debug!("Failed to connect to {}: {}", ssid, e);
            return DriverError::execution(format!("Failed to connect to WiFi: {}", e));
        })?;
        // Wait for connection to establish
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        info!("Connected to WiFi network: {}", ssid);
        return Ok(format!("Connected to WiFi network: {}", ssid));
    }
}
