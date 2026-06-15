//! WiFi auto connect toggle skill - enable/disable auto connect to known networks

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct WifiAutoConnectToggleSkill;

#[async_trait::async_trait]
impl Skill for WifiAutoConnectToggleSkill {
    fn name(&self) -> &str {
        "wifi_auto_connect_toggle"
    }

    fn description(&self) -> &str {
        "Enable or disable automatic connection to known WiFi networks"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to control whether the device automatically connects to saved WiFi networks when in range."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "enabled".to_string(),
                param_type: "boolean".to_string(),
                description: "Enable (true) or disable (false) auto-connect".to_string(),
                required: true,
                default: None,
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "ssid".to_string(),
                param_type: "string".to_string(),
                description: "Specific SSID to configure (default: all networks)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("MyWiFi".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_auto_connect_toggle",
            "parameters": {
                "enabled": false
            }
        })
    }

    fn example_output(&self) -> String {
        "Auto-connect for WiFi disabled".to_string()
    }

    fn category(&self) -> &str {
        "wifi"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let enabled = parameters
            .get("enabled")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow::anyhow!("Missing 'enabled' parameter"))?;
        let _ssid = parameters
            .get("ssid")
            .and_then(|v| v.as_str());
        #[cfg(target_os = "windows")]
        {
            let value = if enabled { "yes" } else { "no" };
            if let Some(ssid) = _ssid {
                Command::new("netsh")
                    .args(["wlan", "set", "profile", "parameter", "name=", ssid, "connectionmode=", value])
                    .output()?;
            } else {
                // For all profiles
                let output = Command::new("netsh")
                    .args(["wlan", "show", "profiles"])
                    .output()?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains(":") {
                        if let Some(profile) = line.split(':').nth(1) {
                            let profile = profile.trim();
                            if !profile.is_empty() {
                                let _ = Command::new("netsh")
                                    .args(["wlan", "set", "profile", "parameter", "name=", profile, "connectionmode=", value])
                                    .output();
                            }
                        }
                    }
                }
            }
        }
        #[cfg(target_os = "linux")]
        {
            let value = if enabled { "yes" } else { "no" };
            if let Some(ssid) = _ssid {
                Command::new("nmcli")
                    .args(["connection", "modify", ssid, "802-11-wireless.mode", "infrastructure"])
                    .output()?;
                Command::new("nmcli")
                    .args(["connection", "modify", ssid, "connection.autoconnect", value])
                    .output()?;
            } else {
                Command::new("nmcli")
                    .args(["networking", "connectivity", if enabled { "on" } else { "off" }])
                    .output()?;
            }
        }
        let status = if enabled { "enabled" } else { "disabled" };
        Ok(format!("Auto-connect for WiFi {}", status))
    }
}