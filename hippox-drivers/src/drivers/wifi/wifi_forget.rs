//! WiFi forget skill - forget/delete a saved WiFi network
use super::common::forget_wifi;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for forgetting a saved WiFi network
#[derive(Debug)]
pub struct WifiForgetDriver;
#[async_trait::async_trait]
impl Driver for WifiForgetDriver {
    fn name(&self) -> &str {
        return "wifi_forget";
    }
    fn description(&self) -> &str {
        return "Forget/delete a saved WiFi network profile";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to remove a saved WiFi network from the system. The device will no longer automatically connect to this network.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "ssid".to_string(),
            param_type: "string".to_string(),
            description: "WiFi network name (SSID) to forget".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("GuestWiFi".to_string())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_forget",
            "parameters": {
                "ssid": "GuestWiFi"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Forgot WiFi network: GuestWiFi".to_string();
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
        debug!("Executing wifi_forget driver");
        let ssid = parameters.get("ssid").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'ssid' parameter");
            return DriverError::missing_parameter("ssid");
        })?;
        forget_wifi(ssid).map_err(|e| {
            debug!("Failed to forget WiFi network {}: {}", ssid, e);
            return DriverError::execution(format!("Failed to forget WiFi network: {}", e));
        })?;
        info!("Forgot WiFi network: {}", ssid);
        return Ok(format!("Forgot WiFi network: {}", ssid));
    }
}
