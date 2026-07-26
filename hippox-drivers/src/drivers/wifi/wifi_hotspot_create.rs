//! WiFi hotspot create skill - create mobile hotspot (soft AP mode)
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for creating a mobile hotspot
#[derive(Debug)]
pub struct WifiHotspotCreateDriver;
#[async_trait::async_trait]
impl Driver for WifiHotspotCreateDriver {
    fn name(&self) -> &str {
        return "wifi_hotspot_create";
    }
    fn description(&self) -> &str {
        return "Create a mobile hotspot (soft AP mode) to share internet connection";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to turn your computer into a WiFi hotspot. Requires administrator privileges on some platforms.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "ssid".to_string(),
                param_type: "string".to_string(),
                description: "Hotspot network name (SSID)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("MyHotspot".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Hotspot password (min 8 characters)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("password123".to_string())),
                enum_values: None,
            },
        ];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_hotspot_create",
            "parameters": {
                "ssid": "MyHotspot",
                "password": "password123"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Hotspot 'MyHotspot' created and started".to_string();
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
        debug!("Executing wifi_hotspot_create driver");
        let ssid = parameters.get("ssid").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'ssid' parameter");
            return DriverError::missing_parameter("ssid");
        })?;
        let password = parameters.get("password").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'password' parameter");
            return DriverError::missing_parameter("password");
        })?;
        if password.len() < 8 {
            debug!("Password must be at least 8 characters");
            return Err(DriverError::validation("password", "Password must be at least 8 characters"));
        }
        #[cfg(target_os = "windows")]
        {
            Command::new("netsh").args(["wlan", "set", "hostednetwork", "mode=allow", "ssid=", ssid, "key=", password]).output().map_err(|e| {
                debug!("Failed to set hostednetwork: {}", e);
                return DriverError::execution(format!("Failed to set hostednetwork: {}", e));
            })?;
            Command::new("netsh").args(["wlan", "start", "hostednetwork"]).output().map_err(|e| {
                debug!("Failed to start hostednetwork: {}", e);
                return DriverError::execution(format!("Failed to start hostednetwork: {}", e));
            })?;
        }
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("nmcli").args(["device", "wifi", "hotspot", "ifname", "wlan0", "ssid", ssid, "password", password]).output();
            if output.is_err() {
                debug!("Hotspot creation requires 'nmcli' or 'create_ap' tool");
                return Err(DriverError::execution("Hotspot creation requires 'nmcli' or 'create_ap' tool"));
            }
        }
        #[cfg(target_os = "macos")]
        {
            debug!("Hotspot creation on macOS requires System Preferences configuration");
            return Err(DriverError::execution("Hotspot creation on macOS requires System Preferences configuration"));
        }
        info!("Hotspot '{}' created and started", ssid);
        return Ok(format!("Hotspot '{}' created and started", ssid));
    }
}
