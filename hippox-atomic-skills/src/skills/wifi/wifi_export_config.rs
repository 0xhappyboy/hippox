//! WiFi export config skill - export WiFi configuration for backup

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;

use super::common::list_saved_networks;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WifiExportConfigSkill;

#[async_trait::async_trait]
impl Skill for WifiExportConfigSkill {
    fn name(&self) -> &str {
        "wifi_export_config"
    }

    fn description(&self) -> &str {
        "Export WiFi configuration (saved networks and settings) to a file for backup"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to backup all saved WiFi networks. The exported file can be imported later to restore settings."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "file_path".to_string(),
                param_type: "string".to_string(),
                description:
                    "Path to save the exported configuration file (default: ./wifi_backup.json)"
                        .to_string(),
                required: false,
                default: Some(Value::String("./wifi_backup.json".to_string())),
                example: Some(Value::String("/backup/wifi_config.json".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "include_passwords".to_string(),
                param_type: "boolean".to_string(),
                description: "Include passwords in export (default: false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_export_config",
            "parameters": {
                "file_path": "./wifi_backup.json",
                "include_passwords": false
            }
        })
    }

    fn example_output(&self) -> String {
        "WiFi configuration exported to: ./wifi_backup.json (3 networks saved)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Wifi
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let file_path = parameters
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("./wifi_backup.json");

        let include_passwords = parameters
            .get("include_passwords")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let networks = list_saved_networks()?;

        let mut export_data = Vec::new();

        for network in &networks {
            let mut network_info = serde_json::json!({
                "ssid": network.ssid,
                "encryption_type": network.encryption_type,
            });

            if include_passwords {
                if let Some(password) = get_network_password(&network.ssid)? {
                    network_info["password"] = json!(password);
                }
            }

            export_data.push(network_info);
        }

        let export_json = json!({
            "export_date": chrono::Local::now().to_rfc3339(),
            "version": "1.0",
            "networks": export_data,
        });

        let json_string = serde_json::to_string_pretty(&export_json)?;
        let mut file = File::create(file_path)?;
        file.write_all(json_string.as_bytes())?;

        Ok(format!(
            "WiFi configuration exported to: {} ({} networks saved)",
            file_path,
            networks.len()
        ))
    }
}

// Helper function to get network password
fn get_network_password(ssid: &str) -> Result<Option<String>> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("netsh")
            .args(["wlan", "show", "profile", "name=", ssid, "key=clear"])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.contains("Key Content") || line.contains("关键内容") {
                if let Some(pwd) = line.split(':').nth(1) {
                    let pwd = pwd.trim();
                    if !pwd.is_empty() {
                        return Ok(Some(pwd.to_string()));
                    }
                }
            }
        }
    }

    Ok(None)
}
