//! WiFi priority set skill - set connection priority for saved networks

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

use super::common::list_saved_networks;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WifiPrioritySetSkill;

#[async_trait::async_trait]
impl Skill for WifiPrioritySetSkill {
    fn name(&self) -> &str {
        "wifi_priority_set"
    }

    fn description(&self) -> &str {
        "Set connection priority order for saved WiFi networks"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to control which WiFi network your device connects to first when multiple known networks are in range. Higher priority networks are preferred."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "priority_list".to_string(),
            param_type: "array".to_string(),
            description: "List of SSIDs in order of priority (first = highest priority)"
                .to_string(),
            required: true,
            default: None,
            example: Some(json!(["MyWiFi", "GuestWiFi", "OfficeNet"])),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_priority_set",
            "parameters": {
                "priority_list": ["MyWiFi", "GuestWiFi", "OfficeNet"]
            }
        })
    }

    fn example_output(&self) -> String {
        "WiFi priority set: MyWiFi (highest) > GuestWiFi > OfficeNet".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Wifi
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let priority_list = parameters
            .get("priority_list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing 'priority_list' parameter"))?;

        let ssids: Vec<String> = priority_list
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        if ssids.is_empty() {
            anyhow::bail!("priority_list must contain at least one SSID");
        }

        set_network_priority(&ssids)?;

        let priority_display: Vec<String> = ssids
            .iter()
            .enumerate()
            .map(|(i, ssid)| {
                if i == 0 {
                    format!("{} (highest)", ssid)
                } else {
                    ssid.clone()
                }
            })
            .collect();

        Ok(format!(
            "WiFi priority set: {}",
            priority_display.join(" > ")
        ))
    }
}

#[cfg(target_os = "windows")]
fn set_network_priority(ssids: &[String]) -> Result<()> {
    // Windows uses profile priority via XML
    let output = Command::new("netsh")
        .args(["wlan", "show", "profiles"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut current_profiles: Vec<String> = Vec::new();
    for line in stdout.lines() {
        if line.contains(":") && !line.contains("All User Profile") {
            if let Some(profile) = line.split(':').nth(1) {
                let profile = profile.trim();
                if !profile.is_empty() && ssids.contains(&profile.to_string()) {
                    current_profiles.push(profile.to_string());
                }
            }
        }
    }

    // Set priority by reordering profiles
    for (priority, ssid) in ssids.iter().enumerate() {
        if current_profiles.contains(ssid) {
            Command::new("netsh")
                .args([
                    "wlan",
                    "set",
                    "profile",
                    "order",
                    "name=",
                    ssid,
                    "priority=",
                    &priority.to_string(),
                ])
                .output()?;
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn set_network_priority(ssids: &[String]) -> Result<()> {
    // Linux uses NetworkManager connection autoconnect-priority
    for (priority, ssid) in ssids.iter().enumerate() {
        let priority_value = (ssids.len() - priority) * 10;

        // Find connection name (might be different from SSID)
        let output = Command::new("nmcli")
            .args(["-t", "-f", "NAME,TYPE", "connection", "show"])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 && parts[1] == "802-11-wireless" {
                let conn_name = parts[0];
                if conn_name.contains(ssid) || ssid.contains(conn_name) {
                    Command::new("nmcli")
                        .args([
                            "connection",
                            "modify",
                            conn_name,
                            "connection.autoconnect-priority",
                            &priority_value.to_string(),
                        ])
                        .output()?;
                    break;
                }
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn set_network_priority(ssids: &[String]) -> Result<()> {
    // macOS uses preferred networks order
    let service_name = get_wifi_service_name()?;

    // Get existing preferred networks
    let output = Command::new("networksetup")
        .args(["-listpreferredwirelessnetworks", &service_name])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Build new order list
    let mut new_order = Vec::new();
    for ssid in ssids {
        new_order.push(ssid.as_str());
    }

    // Add any existing networks not in priority list at the end
    for line in stdout.lines().skip(1) {
        let line_ssid = line.trim_start_matches('*').trim();
        if !ssids.contains(&line_ssid.to_string()) && !new_order.contains(&line_ssid) {
            new_order.push(line_ssid);
        }
    }

    // Remove all existing preferred networks
    let output = Command::new("networksetup")
        .args(["-listpreferredwirelessnetworks", &service_name])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines().skip(1) {
        let ssid = line.trim_start_matches('*').trim();
        let _ = Command::new("networksetup")
            .args(["-removepreferredwirelessnetwork", &service_name, ssid])
            .output();
    }

    // Add networks in new priority order
    for ssid in new_order {
        Command::new("networksetup")
            .args(["-addpreferredwirelessnetwork", &service_name, ssid, "0"])
            .output()?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn get_wifi_service_name() -> Result<String> {
    let output = Command::new("networksetup")
        .args(["-listallhardwareports"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let lines: Vec<&str> = stdout.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.contains("Hardware Port: Wi-Fi") || line.contains("Hardware Port: AirPort") {
            if i + 1 < lines.len() && lines[i + 1].contains("Device:") {
                if let Some(device) = lines[i + 1].split(':').nth(1) {
                    return Ok(device.trim().to_string());
                }
            }
        }
    }

    Ok("Wi-Fi".to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn set_network_priority(_ssids: &[String]) -> Result<()> {
    anyhow::bail!("Priority setting not implemented on this platform")
}
