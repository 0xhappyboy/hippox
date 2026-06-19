//! WiFi connect hidden skill - connect to hidden SSID network

use super::common::connect_wifi;
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
pub struct WifiConnectHiddenDriver;

#[async_trait::async_trait]
impl Driver for WifiConnectHiddenDriver {
    fn name(&self) -> &str {
        "wifi_connect_hidden"
    }

    fn description(&self) -> &str {
        "Connect to a hidden WiFi network (non-broadcasting SSID)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to connect to networks that don't broadcast their SSID. You must know the exact SSID and password."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_connect_hidden",
            "parameters": {
                "ssid": "HiddenNetwork",
                "password": "secret123"
            }
        })
    }

    fn example_output(&self) -> String {
        "Connected to hidden WiFi network: HiddenNetwork".to_string()
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
        let password = parameters
            .get("password")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'password' parameter"))?;
        // Hidden network connection is the same as regular, just not visible in scan
        connect_wifi(ssid, Some(password))?;
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        Ok(format!("Connected to hidden WiFi network: {}", ssid))
    }
}
