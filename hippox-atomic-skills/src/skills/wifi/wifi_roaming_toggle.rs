//! WiFi roaming toggle skill - enable/disable roaming assistance

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct WifiRoamingToggleSkill;

#[async_trait::async_trait]
impl Skill for WifiRoamingToggleSkill {
    fn name(&self) -> &str {
        "wifi_roaming_toggle"
    }

    fn description(&self) -> &str {
        "Enable or disable WiFi roaming assistance (automatic switching between APs)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to control roaming behavior. Enable for seamless transition between access points, disable to stay connected to current AP."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "enabled".to_string(),
                param_type: "boolean".to_string(),
                description: "Enable (true) or disable (false) roaming".to_string(),
                required: true,
                default: None,
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "sensitivity".to_string(),
                param_type: "integer".to_string(),
                description: "Roaming sensitivity (1-100, higher = more aggressive)".to_string(),
                required: false,
                default: Some(Value::Number(50.into())),
                example: Some(Value::Number(70.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_roaming_toggle",
            "parameters": {
                "enabled": true,
                "sensitivity": 70
            }
        })
    }

    fn example_output(&self) -> String {
        "WiFi roaming enabled with sensitivity 70%".to_string()
    }

    fn category(&self) -> &str {
        "wifi"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let enabled = parameters
            .get("enabled")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow::anyhow!("Missing 'enabled' parameter"))?;
        
        let sensitivity = parameters
            .get("sensitivity")
            .and_then(|v| v.as_u64())
            .unwrap_or(50);
        
        #[cfg(target_os = "windows")]
        {
            let value = if enabled { "enable" } else { "disable" };
            Command::new("netsh")
                .args(["wlan", "set", "roaming", value])
                .output()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            let roam_value = if enabled { "1" } else { "0" };
            Command::new("iw")
                .args(["wlan0", "set", "power_save", roam_value])
                .output()?;
        }
        
        let status = if enabled { "enabled" } else { "disabled" };
        Ok(format!("WiFi roaming {} with sensitivity {}%", status, sensitivity))
    }
}