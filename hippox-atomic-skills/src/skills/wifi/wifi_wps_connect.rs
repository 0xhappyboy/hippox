//! WiFi WPS connect skill - connect using WPS button

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct WifiWpsConnectSkill;

#[async_trait::async_trait]
impl Skill for WifiWpsConnectSkill {
    fn name(&self) -> &str {
        "wifi_wps_connect"
    }

    fn description(&self) -> &str {
        "Connect to a WiFi network using WPS (WiFi Protected Setup) button method"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to connect to a router using WPS. Press the WPS button on your router within 2 minutes of calling this skill."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "timeout_secs".to_string(),
                param_type: "integer".to_string(),
                description: "Timeout in seconds to wait for WPS connection (default: 120)".to_string(),
                required: false,
                default: Some(Value::Number(120.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_wps_connect",
            "parameters": {
                "timeout_secs": 120
            }
        })
    }

    fn example_output(&self) -> String {
        "WPS connection initiated. Please press the WPS button on your router. Connected successfully!".to_string()
    }

    fn category(&self) -> &str {
        "wifi"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let timeout = parameters
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(120);
        
        #[cfg(target_os = "windows")]
        {
            Command::new("netsh")
                .args(["wlan", "wps", "start", "pin"])
                .output()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("wpa_cli")
                .args(["wps_pbc"])
                .output();
            
            if output.is_err() {
                anyhow::bail!("WPS not supported. Ensure wpa_supplicant is running.");
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            anyhow::bail!("WPS is not supported on macOS");
        }
        
        Ok(format!("WPS connection initiated. Please press the WPS button on your router within {} seconds.", timeout))
    }
}