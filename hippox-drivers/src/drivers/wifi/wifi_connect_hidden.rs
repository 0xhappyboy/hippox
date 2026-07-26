//! WiFi connect hidden skill - connect to hidden SSID network
use super::common::connect_wifi;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for connecting to hidden WiFi networks
#[derive(Debug)]
pub struct WifiConnectHiddenDriver;
#[async_trait::async_trait]
impl Driver for WifiConnectHiddenDriver {
    fn name(&self) -> &str {
        return "wifi_connect_hidden";
    }
    fn description(&self) -> &str {
        return "Connect to a hidden WiFi network (non-broadcasting SSID)";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to connect to networks that don't broadcast their SSID. You must know the exact SSID and password.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "ssid".to_string(),
                param_type: "string".to_string(),
                description: "Hidden WiFi network name (SSID)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("HiddenNetwork".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "WiFi password".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("secret123".to_string())),
                enum_values: None,
            },
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_connect_hidden",
            "parameters": {
                "ssid": "HiddenNetwork",
                "password": "secret123"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Connected to hidden WiFi network: HiddenNetwork".to_string();
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
        debug!("Executing wifi_connect_hidden driver");
        let ssid = parameters.get("ssid").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'ssid' parameter");
            return DriverError::missing_parameter("ssid");
        })?;
        let password = parameters.get("password").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'password' parameter");
            return DriverError::missing_parameter("password");
        })?;
        debug!("Connecting to hidden WiFi network: {}", ssid);
        connect_wifi(ssid, Some(password)).map_err(|e| {
            debug!("Failed to connect to hidden network {}: {}", ssid, e);
            return DriverError::execution(format!("Failed to connect to hidden WiFi: {}", e));
        })?;
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        info!("Connected to hidden WiFi network: {}", ssid);
        return Ok(format!("Connected to hidden WiFi network: {}", ssid));
    }
}
