//! WiFi forget skill - forget/delete a saved WiFi network

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::forget_wifi;
use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};

#[derive(Debug)]
pub struct WifiForgetDriver;

#[async_trait::async_trait]
impl Driver for WifiForgetDriver {
    fn name(&self) -> &str {
        "wifi_forget"
    }

    fn description(&self) -> &str {
        "Forget/delete a saved WiFi network profile"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to remove a saved WiFi network from the system. The device will no longer automatically connect to this network."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "ssid".to_string(),
            param_type: "string".to_string(),
            description: "WiFi network name (SSID) to forget".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("GuestWiFi".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_forget",
            "parameters": {
                "ssid": "GuestWiFi"
            }
        })
    }

    fn example_output(&self) -> String {
        "Forgot WiFi network: GuestWiFi".to_string()
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
        let ssid = parameters
            .get("ssid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'ssid' parameter"))?;
        forget_wifi(ssid)?;
        Ok(format!("Forgot WiFi network: {}", ssid))
    }
}
