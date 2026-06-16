//! WiFi MAC address get skill - get current MAC address

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WifiMacAddressGetSkill;

#[async_trait::async_trait]
impl Skill for WifiMacAddressGetSkill {
    fn name(&self) -> &str {
        "wifi_mac_address_get"
    }

    fn description(&self) -> &str {
        "Get the current WiFi adapter MAC address"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to retrieve the hardware MAC address of your WiFi adapter."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "interface".to_string(),
            param_type: "string".to_string(),
            description: "Interface name (default: auto-detect)".to_string(),
            required: false,
            default: None,
            example: Some(Value::String("wlan0".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_mac_address_get"
        })
    }

    fn example_output(&self) -> String {
        "MAC Address: 00:11:22:33:44:55".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Wifi
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let interface = parameters
            .get("interface")
            .and_then(|v| v.as_str())
            .unwrap_or("wlan0");

        let mac = get_mac_address(interface)?;

        Ok(format!("MAC Address: {}", mac))
    }
}

// Helper function to get MAC address
fn get_mac_address(interface: &str) -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("ip")
            .args(["link", "show", interface])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.contains("link/ether") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Ok(parts[1].to_string());
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("getmac").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.contains("Wi-Fi") || line.contains("WLAN") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 1 {
                    return Ok(parts[0].to_string());
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("ifconfig").args([interface]).output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.contains("ether") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Ok(parts[1].to_string());
                }
            }
        }
    }

    Ok("Unknown".to_string())
}
