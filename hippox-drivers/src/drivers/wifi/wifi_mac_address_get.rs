//! WiFi MAC address get skill - get current MAC address
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for getting WiFi MAC address
#[derive(Debug)]
pub struct WifiMacAddressGetDriver;
#[async_trait::async_trait]
impl Driver for WifiMacAddressGetDriver {
    fn name(&self) -> &str {
        return "wifi_mac_address_get";
    }
    fn description(&self) -> &str {
        return "Get the current WiFi adapter MAC address";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to retrieve the hardware MAC address of your WiFi adapter.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "interface".to_string(),
            param_type: "string".to_string(),
            description: "Interface name (default: auto-detect)".to_string(),
            required: false,
            default: None,
            example: Some(Value::String("wlan0".to_string())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_mac_address_get"
        }));
    }
    fn example_output(&self) -> String {
        return "MAC Address: 00:11:22:33:44:55".to_string();
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
        debug!("Executing wifi_mac_address_get driver");
        let interface = parameters.get("interface").and_then(|v| v.as_str()).unwrap_or("wlan0");
        let mac = get_mac_address(interface).map_err(|e| {
            debug!("Failed to get MAC address: {}", e);
            return DriverError::execution(format!("Failed to get MAC address: {}", e));
        })?;
        info!("MAC Address: {}", mac);
        return Ok(format!("MAC Address: {}", mac));
    }
}
/// Helper function to get MAC address
fn get_mac_address(interface: &str) -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("ip").args(["link", "show", interface]).output().map_err(|e| format!("Failed to get interface: {}", e))?;
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
        let output = Command::new("getmac").output().map_err(|e| format!("Failed to get MAC: {}", e))?;
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
        let output = Command::new("ifconfig").args([interface]).output().map_err(|e| format!("Failed to get interface: {}", e))?;
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
    return Ok("Unknown".to_string());
}
