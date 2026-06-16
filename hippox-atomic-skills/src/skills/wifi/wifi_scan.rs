//! WiFi scan skill - scan for nearby WiFi networks

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{WiFiNetwork, scan_wifi_networks};
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WifiScanSkill;

#[async_trait::async_trait]
impl Skill for WifiScanSkill {
    fn name(&self) -> &str {
        "wifi_scan"
    }

    fn description(&self) -> &str {
        "Scan for nearby WiFi networks and return SSID, signal strength, and encryption type"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to discover available WiFi networks in the area. Returns a list of networks sorted by signal strength."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "timeout_secs".to_string(),
            param_type: "integer".to_string(),
            description: "Scan timeout in seconds (default: 10)".to_string(),
            required: false,
            default: Some(Value::Number(10.into())),
            example: Some(Value::Number(15.into())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_scan",
            "parameters": {
                "timeout_secs": 10
            }
        })
    }

    fn example_output(&self) -> String {
        "Found 5 networks:\n1. MyWiFi (Signal: 85%, Security: WPA2-Personal)\n2. GuestWiFi (Signal: 45%, Security: Open)\n3. OfficeNet (Signal: 30%, Security: WPA2-Enterprise)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Wifi
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let timeout = parameters
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);

        tokio::time::sleep(std::time::Duration::from_secs(timeout)).await;

        let networks = scan_wifi_networks()?;

        if networks.is_empty() {
            return Ok("No WiFi networks found".to_string());
        }

        let mut result = format!("Found {} networks:\n", networks.len());
        for (i, network) in networks.iter().enumerate() {
            let connected_marker = if network.is_connected {
                " [CONNECTED]"
            } else {
                ""
            };
            result.push_str(&format!(
                "{}. {}{} (Signal: {}%, Security: {})",
                i + 1,
                network.ssid,
                connected_marker,
                network.signal_strength,
                network.encryption_type
            ));

            if let Some(bssid) = &network.bssid {
                result.push_str(&format!(", BSSID: {}", bssid));
            }
            if let Some(channel) = network.channel {
                result.push_str(&format!(", Channel: {}", channel));
            }
            result.push('\n');
        }

        Ok(result)
    }
}
