//! WiFi hotspot stop skill - stop mobile hotspot

use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

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

    fn category(&self) -> SkillCategory {
        SkillCategory::Wifi
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
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
