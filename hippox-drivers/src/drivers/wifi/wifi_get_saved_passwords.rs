//! WiFi get saved passwords skill - retrieve saved WiFi passwords
use super::common::list_saved_networks;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for retrieving saved WiFi passwords
#[derive(Debug)]
pub struct WifiGetSavedPasswordsDriver;
#[async_trait::async_trait]
impl Driver for WifiGetSavedPasswordsDriver {
    fn name(&self) -> &str {
        return "wifi_get_saved_passwords";
    }
    fn description(&self) -> &str {
        return "Get saved WiFi passwords for networks the device has connected to";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to retrieve saved WiFi passwords. May require administrator privileges.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "ssid".to_string(),
            param_type: "string".to_string(),
            description: "Specific SSID to get password for (omit for all networks)".to_string(),
            required: false,
            default: None,
            example: Some(Value::String("MyWiFi".to_string())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_get_saved_passwords",
            "parameters": {
                "ssid": "MyWiFi"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Password for MyWiFi: password123".to_string();
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
        debug!("Executing wifi_get_saved_passwords driver");
        let specific_ssid = parameters.get("ssid").and_then(|v| v.as_str());
        let mut result = String::new();
        #[cfg(target_os = "windows")]
        {
            if let Some(ssid) = specific_ssid {
                let output = Command::new("netsh").args(["wlan", "show", "profile", "name=", ssid, "key=clear"]).output().map_err(|e| {
                    debug!("Failed to get password for {}: {}", ssid, e);
                    return DriverError::execution(format!("Failed to get password: {}", e));
                })?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("Key Content") || line.contains("关键内容") {
                        if let Some(pwd) = line.split(':').nth(1) {
                            result = format!("Password for {}: {}", ssid, pwd.trim());
                            break;
                        }
                    }
                }
                if result.is_empty() {
                    result = format!("No password found for {}", ssid);
                }
            } else {
                let networks = list_saved_networks().map_err(|e| {
                    debug!("Failed to list saved networks: {}", e);
                    return DriverError::execution(format!("Failed to list saved networks: {}", e));
                })?;
                for network in networks {
                    let output =
                        Command::new("netsh").args(["wlan", "show", "profile", "name=", &network.ssid, "key=clear"]).output().map_err(|e| {
                            debug!("Failed to get password for {}: {}", network.ssid, e);
                            return DriverError::execution(format!("Failed to get password: {}", e));
                        })?;
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        if line.contains("Key Content") || line.contains("关键内容") {
                            if let Some(pwd) = line.split(':').nth(1) {
                                result.push_str(&format!("{}: {}\n", network.ssid, pwd.trim()));
                                break;
                            }
                        }
                    }
                }
            }
        }
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            result = "Password retrieval on this platform requires root privileges. Please run with sudo.".to_string();
        }
        if result.is_empty() {
            result = "No saved passwords found".to_string();
        }
        info!("Retrieved saved passwords");
        return Ok(result);
    }
}
