//! WiFi status skill - get current WiFi connection status

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_wifi_status;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WifiStatusSkill;

#[async_trait::async_trait]
impl Skill for WifiStatusSkill {
    fn name(&self) -> &str {
        "wifi_status"
    }

    fn description(&self) -> &str {
        "Get current WiFi connection status including SSID, signal strength, IP address, and channel"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check if WiFi is connected and get detailed connection information."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "verbose".to_string(),
            param_type: "boolean".to_string(),
            description: "Show detailed information including BSSID and link speed".to_string(),
            required: false,
            default: Some(Value::Bool(false)),
            example: Some(Value::Bool(true)),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_status",
            "parameters": {
                "verbose": true
            }
        })
    }

    fn example_output(&self) -> String {
        "WiFi Status:\n- Connected: Yes\n- SSID: MyWiFi\n- IP Address: 192.168.1.100\n- Signal Strength: 85%\n- Channel: 6 (2.4GHz)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Wifi
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let verbose = parameters
            .get("verbose")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let status = get_wifi_status()?;

        if !status.connected {
            return Ok("WiFi is not connected".to_string());
        }

        let mut result = String::from("WiFi Status:\n");
        result.push_str(&format!("- Connected: Yes\n"));

        if let Some(ssid) = &status.ssid {
            result.push_str(&format!("- SSID: {}\n", ssid));
        }
        if let Some(ip) = &status.ip_address {
            result.push_str(&format!("- IP Address: {}\n", ip));
        }
        if let Some(signal) = status.signal_strength {
            result.push_str(&format!("- Signal Strength: {}%\n", signal));
        }
        if let Some(channel) = status.channel {
            let freq = if channel <= 14 {
                "2.4GHz"
            } else if channel <= 64 {
                "5GHz"
            } else {
                "6GHz"
            };
            result.push_str(&format!("- Channel: {} ({})\n", channel, freq));
        }

        if verbose {
            if let Some(bssid) = &status.bssid {
                result.push_str(&format!("- BSSID: {}\n", bssid));
            }
            if let Some(speed) = status.link_speed {
                result.push_str(&format!("- Link Speed: {} Mbps\n", speed));
            }
        }

        Ok(result)
    }
}
