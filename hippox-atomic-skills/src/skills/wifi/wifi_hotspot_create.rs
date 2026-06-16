//! WiFi hotspot create skill - create mobile hotspot (soft AP mode)

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WifiHotspotCreateSkill;

#[async_trait::async_trait]
impl Skill for WifiHotspotCreateSkill {
    fn name(&self) -> &str {
        "wifi_hotspot_create"
    }

    fn description(&self) -> &str {
        "Create a mobile hotspot (soft AP mode) to share internet connection"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to turn your computer into a WiFi hotspot. Requires administrator privileges on some platforms."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "ssid".to_string(),
                param_type: "string".to_string(),
                description: "Hotspot network name (SSID)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("MyHotspot".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Hotspot password (min 8 characters)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("password123".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_hotspot_create",
            "parameters": {
                "ssid": "MyHotspot",
                "password": "password123"
            }
        })
    }

    fn example_output(&self) -> String {
        "Hotspot 'MyHotspot' created and started".to_string()
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

        if password.len() < 8 {
            anyhow::bail!("Password must be at least 8 characters");
        }

        #[cfg(target_os = "windows")]
        {
            Command::new("netsh")
                .args([
                    "wlan",
                    "set",
                    "hostednetwork",
                    "mode=allow",
                    "ssid=",
                    ssid,
                    "key=",
                    password,
                ])
                .output()?;
            Command::new("netsh")
                .args(["wlan", "start", "hostednetwork"])
                .output()?;
        }

        #[cfg(target_os = "linux")]
        {
            // Use create_ap or nmcli
            let output = Command::new("nmcli")
                .args([
                    "device", "wifi", "hotspot", "ifname", "wlan0", "ssid", ssid, "password",
                    password,
                ])
                .output();

            if output.is_err() {
                anyhow::bail!("Hotspot creation requires 'nmcli' or 'create_ap' tool");
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS requires Internet Sharing to be configured manually
            anyhow::bail!("Hotspot creation on macOS requires System Preferences configuration");
        }

        Ok(format!("Hotspot '{}' created and started", ssid))
    }
}
