//! WiFi get channel skill - get current WiFi channel and frequency

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use super::common::get_wifi_status;

#[derive(Debug)]
pub struct WifiGetChannelSkill;

#[async_trait::async_trait]
impl Skill for WifiGetChannelSkill {
    fn name(&self) -> &str {
        "wifi_get_channel"
    }

    fn description(&self) -> &str {
        "Get the current WiFi channel and frequency band (2.4GHz/5GHz/6GHz)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see what channel your WiFi is using. Useful for diagnosing interference issues."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_get_channel"
        })
    }

    fn example_output(&self) -> String {
        "Current channel: 6 (2.4GHz)".to_string()
    }

    fn category(&self) -> &str {
        "wifi"
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let status = get_wifi_status()?;
        
        if !status.connected {
            return Ok("Not connected to WiFi".to_string());
        }
        
        if let Some(channel) = status.channel {
            let freq = if channel <= 14 {
                "2.4GHz"
            } else if channel <= 64 {
                "5GHz"
            } else {
                "6GHz"
            };
            Ok(format!("Current channel: {} ({})", channel, freq))
        } else {
            Ok("Channel information not available".to_string())
        }
    }
}