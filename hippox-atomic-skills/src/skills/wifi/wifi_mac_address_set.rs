//! WiFi MAC address set skill - set/spoof MAC address

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WifiMacAddressSetSkill;

#[async_trait::async_trait]
impl Skill for WifiMacAddressSetSkill {
    fn name(&self) -> &str {
        "wifi_mac_address_set"
    }

    fn description(&self) -> &str {
        "Set or spoof the WiFi adapter MAC address"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to change your MAC address for privacy or to bypass MAC filters. May require administrator privileges."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "mac".to_string(),
                param_type: "string".to_string(),
                description:
                    "New MAC address (format: XX:XX:XX:XX:XX:XX) or 'random' for random MAC"
                        .to_string(),
                required: true,
                default: None,
                example: Some(Value::String("00:11:22:33:44:55".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "interface".to_string(),
                param_type: "string".to_string(),
                description: "Interface name (default: auto-detect)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("wlan0".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_mac_address_set",
            "parameters": {
                "mac": "random"
            }
        })
    }

    fn example_output(&self) -> String {
        "MAC address set to random: 3a:2f:8e:1c:4b:7d".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Wifi
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let mac_input = parameters
            .get("mac")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'mac' parameter"))?;

        let interface = parameters
            .get("interface")
            .and_then(|v| v.as_str())
            .unwrap_or("wlan0");

        let new_mac = if mac_input == "random" {
            // Generate random MAC (locally administered, unicast)
            format!(
                "02:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                rand::random::<u8>(),
                rand::random::<u8>(),
                rand::random::<u8>(),
                rand::random::<u8>(),
                rand::random::<u8>()
            )
        } else {
            mac_input.to_string()
        };

        #[cfg(target_os = "linux")]
        {
            // Bring interface down, change MAC, bring up
            Command::new("sudo")
                .args(["ip", "link", "set", interface, "down"])
                .output()?;
            Command::new("sudo")
                .args(["ip", "link", "set", interface, "address", &new_mac])
                .output()?;
            Command::new("sudo")
                .args(["ip", "link", "set", interface, "up"])
                .output()?;
        }

        #[cfg(target_os = "windows")]
        {
            // Windows requires registry changes or device disable/enable
            anyhow::bail!("MAC address change on Windows requires device manager manipulation");
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("sudo")
                .args(["ifconfig", interface, "ether", &new_mac])
                .output()?;
        }

        Ok(format!("MAC address set to: {}", new_mac))
    }
}
