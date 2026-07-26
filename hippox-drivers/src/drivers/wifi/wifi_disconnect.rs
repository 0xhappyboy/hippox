//! WiFi disconnect skill - disconnect from current WiFi
use super::common::{disconnect_wifi, get_wifi_status};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for disconnecting from current WiFi network
#[derive(Debug)]
pub struct WifiDisconnectDriver;
#[async_trait::async_trait]
impl Driver for WifiDisconnectDriver {
    fn name(&self) -> &str {
        return "wifi_disconnect";
    }
    fn description(&self) -> &str {
        return "Disconnect from the current WiFi network";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to disconnect from the currently connected WiFi network without forgetting it.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_disconnect"
        }));
    }
    fn example_output(&self) -> String {
        return "Disconnected from WiFi network: MyWiFi".to_string();
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
        debug!("Executing wifi_disconnect driver");
        let status = get_wifi_status().map_err(|e| {
            debug!("Failed to get WiFi status: {}", e);
            return DriverError::execution(format!("Failed to get WiFi status: {}", e));
        })?;
        let current_ssid = status.ssid.clone();
        if current_ssid.is_none() {
            info!("Already disconnected from WiFi");
            return Ok("Already disconnected from WiFi".to_string());
        }
        disconnect_wifi().map_err(|e| {
            debug!("Failed to disconnect WiFi: {}", e);
            return DriverError::execution(format!("Failed to disconnect WiFi: {}", e));
        })?;
        info!("Disconnected from WiFi network: {}", current_ssid.as_ref().unwrap());
        return Ok(format!("Disconnected from WiFi network: {}", current_ssid.unwrap()));
    }
}
