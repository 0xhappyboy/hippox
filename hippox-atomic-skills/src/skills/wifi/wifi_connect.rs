//! WiFi connect skill - connect to a WiFi network

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::connect_wifi;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WifiConnectSkill;

#[async_trait::async_trait]
impl Skill for WifiConnectSkill {
    fn name(&self) -> &str {
        "wifi_connect"
    }

    fn description(&self) -> &str {
        "Connect to a WiFi network using SSID and password"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to connect to a WiFi network. Provide the network SSID and password. If the network is open (no password), omit the password parameter."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "ssid".to_string(),
                param_type: "string".to_string(),
                description: "WiFi network name (SSID)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("MyWiFi".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "WiFi password (omit for open networks)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("password123".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_connect",
            "parameters": {
                "ssid": "MyWiFi",
                "password": "password123"
            }
        })
    }

    fn example_output(&self) -> String {
        "Connected to WiFi network: MyWiFi".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Wifi
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let ssid = parameters
            .get("ssid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'ssid' parameter"))?;
        let password = parameters.get("password").and_then(|v| v.as_str());
        connect_wifi(ssid, password)?;
        // Wait for connection to establish
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        Ok(format!("Connected to WiFi network: {}", ssid))
    }
}
