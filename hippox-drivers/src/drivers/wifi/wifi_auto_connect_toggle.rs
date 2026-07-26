//! WiFi auto connect toggle skill - enable/disable auto connect to known networks
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for toggling automatic WiFi connection
#[derive(Debug)]
pub struct WifiAutoConnectToggleDriver;
#[async_trait::async_trait]
impl Driver for WifiAutoConnectToggleDriver {
    fn name(&self) -> &str {
        return "wifi_auto_connect_toggle";
    }
    fn description(&self) -> &str {
        return "Enable or disable automatic connection to known WiFi networks";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to control whether the device automatically connects to saved WiFi networks when in range.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "enabled".to_string(),
                param_type: "boolean".to_string(),
                description: "Enable (true) or disable (false) auto-connect".to_string(),
                required: true,
                default: None,
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "ssid".to_string(),
                param_type: "string".to_string(),
                description: "Specific SSID to configure (default: all networks)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("MyWiFi".to_string())),
                enum_values: None,
            },
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_auto_connect_toggle",
            "parameters": {
                "enabled": false
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Auto-connect for WiFi disabled".to_string();
    }
    fn category(&self) -> DriverCategory {
        return DriverCategory::Wifi;
    }
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing wifi_auto_connect_toggle driver");
        let enabled = parameters.get("enabled").and_then(|v| v.as_bool()).ok_or_else(|| {
            debug!("Missing 'enabled' parameter");
            return DriverError::missing_parameter("enabled");
        })?;
        let _ssid = parameters.get("ssid").and_then(|v| v.as_str());
        #[cfg(target_os = "windows")]
        {
            let value = if enabled { "yes" } else { "no" };
            if let Some(ssid) = _ssid {
                Command::new("netsh").args(["wlan", "set", "profile", "parameter", "name=", ssid, "connectionmode=", value]).output().map_err(
                    |e| {
                        debug!("Failed to set auto-connect for {}: {}", ssid, e);
                        return DriverError::execution(format!("Failed to set auto-connect: {}", e));
                    },
                )?;
            } else {
                // For all profiles
                let output = Command::new("netsh").args(["wlan", "show", "profiles"]).output().map_err(|e| {
                    debug!("Failed to list profiles: {}", e);
                    return DriverError::execution(format!("Failed to list profiles: {}", e));
                })?;
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
                Command::new("nmcli").args(["connection", "modify", ssid, "802-11-wireless.mode", "infrastructure"]).output().map_err(|e| {
                    debug!("Failed to modify connection mode for {}: {}", ssid, e);
                    return DriverError::execution(format!("Failed to modify connection: {}", e));
                })?;
                Command::new("nmcli").args(["connection", "modify", ssid, "connection.autoconnect", value]).output().map_err(|e| {
                    debug!("Failed to set autoconnect for {}: {}", ssid, e);
                    return DriverError::execution(format!("Failed to set autoconnect: {}", e));
                })?;
            } else {
                Command::new("nmcli").args(["networking", "connectivity", if enabled { "on" } else { "off" }]).output().map_err(|e| {
                    debug!("Failed to set networking connectivity: {}", e);
                    return DriverError::execution(format!("Failed to set connectivity: {}", e));
                })?;
            }
        }
        let status = if enabled { "enabled" } else { "disabled" };
        info!("Auto-connect for WiFi {}", status);
        return Ok(format!("Auto-connect for WiFi {}", status));
    }
}
