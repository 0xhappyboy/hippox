//! WiFi band preference get skill - get current frequency band preference

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WifiBandPreferenceGetSkill;

#[async_trait::async_trait]
impl Skill for WifiBandPreferenceGetSkill {
    fn name(&self) -> &str {
        "wifi_band_preference_get"
    }

    fn description(&self) -> &str {
        "Get the current WiFi frequency band preference setting"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see which band (2.4GHz/5GHz/6GHz) is currently preferred."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_band_preference_get"
        })
    }

    fn example_output(&self) -> String {
        "Current band preference: 5GHz".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Wifi
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("netsh")
                .args(["wlan", "show", "settings"])
                .output()?;
            let stdout = String::from_utf8_lossy(&output.stdout);

            for line in stdout.lines() {
                if line.contains("Band") || line.contains("频段") {
                    if let Some(band) = line.split(':').nth(1) {
                        return Ok(format!("Current band preference: {}", band.trim()));
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let output = Command::new("iw").args(["reg", "get"]).output()?;
            let stdout = String::from_utf8_lossy(&output.stdout);

            if stdout.contains("2.4") && stdout.contains("5") {
                return Ok("Current band preference: Auto (2.4GHz and 5GHz)".to_string());
            } else if stdout.contains("2.4") {
                return Ok("Current band preference: 2.4GHz".to_string());
            } else if stdout.contains("5") {
                return Ok("Current band preference: 5GHz".to_string());
            }
        }

        Ok("Current band preference: Auto".to_string())
    }
}
