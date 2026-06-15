//! WiFi hotspot stop skill - stop mobile hotspot

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct WifiHotspotStopSkill;

#[async_trait::async_trait]
impl Skill for WifiHotspotStopSkill {
    fn name(&self) -> &str {
        "wifi_hotspot_stop"
    }

    fn description(&self) -> &str {
        "Stop the mobile hotspot (soft AP mode)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to stop the WiFi hotspot that was previously created."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_hotspot_stop"
        })
    }

    fn example_output(&self) -> String {
        "Hotspot stopped".to_string()
    }

    fn category(&self) -> &str {
        "wifi"
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        #[cfg(target_os = "windows")]
        {
            Command::new("netsh")
                .args(["wlan", "stop", "hostednetwork"])
                .output()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("nmcli")
                .args(["connection", "down", "Hotspot"])
                .output();
        }
        
        Ok("Hotspot stopped".to_string())
    }
}