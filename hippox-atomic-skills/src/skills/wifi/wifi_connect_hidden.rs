//! WiFi connect hidden skill - connect to hidden SSID network

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::connect_wifi;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WifiConnectHiddenSkill;

#[async_trait::async_trait]
impl Skill for WifiConnectHiddenSkill {
    fn name(&self) -> &str {
        "wifi_connect_hidden"
    }

    fn description(&self) -> &str {
        "Connect to a hidden WiFi network (non-broadcasting SSID)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to connect to networks that don't broadcast their SSID. You must know the exact SSID and password."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "ssid".to_string(),
                param_type: "string".to_string(),
                description: "Hidden WiFi network name (SSID)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("HiddenNetwork".to_string())),
                enum_values: None,
            },
            SkillParameter {
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

    fn category(&self) -> SkillCategory {
        SkillCategory::Wifi
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
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
