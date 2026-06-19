//! WiFi disconnect skill - disconnect from current WiFi

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{disconnect_wifi, get_wifi_status};
use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};

#[derive(Debug)]
pub struct WifiDisconnectDriver;

#[async_trait::async_trait]
impl Driver for WifiDisconnectDriver {
    fn name(&self) -> &str {
        "wifi_disconnect"
    }

    fn description(&self) -> &str {
        "Disconnect from the current WiFi network"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to disconnect from the currently connected WiFi network without forgetting it."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_disconnect"
        })
    }

    fn example_output(&self) -> String {
        "Disconnected from WiFi network: MyWiFi".to_string()
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
        let status = get_wifi_status()?;
        let current_ssid = status.ssid.clone();
        if current_ssid.is_none() {
            return Ok("Already disconnected from WiFi".to_string());
        }
        disconnect_wifi()?;
        Ok(format!(
            "Disconnected from WiFi network: {}",
            current_ssid.unwrap()
        ))
    }
}
