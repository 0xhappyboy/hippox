//! WiFi WPS connect skill - connect using WPS button
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for WPS connection
#[derive(Debug)]
pub struct WifiWpsConnectDriver;
#[async_trait::async_trait]
impl Driver for WifiWpsConnectDriver {
    fn name(&self) -> &str {
        return "wifi_wps_connect";
    }
    fn description(&self) -> &str {
        return "Connect to a WiFi network using WPS (WiFi Protected Setup) button method";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to connect to a router using WPS. Press the WPS button on your router within 2 minutes of calling this skill.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "timeout_secs".to_string(),
            param_type: "integer".to_string(),
            description: "Timeout in seconds to wait for WPS connection (default: 120)".to_string(),
            required: false,
            default: Some(Value::Number(120.into())),
            example: Some(Value::Number(60.into())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_wps_connect",
            "parameters": {
                "timeout_secs": 120
            }
        }));
    }
    fn example_output(&self) -> String {
        return "WPS connection initiated. Please press the WPS button on your router. Connected successfully!".to_string();
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
        debug!("Executing wifi_wps_connect driver");
        let timeout = parameters.get("timeout_secs").and_then(|v| v.as_u64()).unwrap_or(120);
        #[cfg(target_os = "windows")]
        {
            Command::new("netsh").args(["wlan", "wps", "start", "pin"]).output().map_err(|e| {
                debug!("Failed to start WPS: {}", e);
                return DriverError::execution(format!("Failed to start WPS: {}", e));
            })?;
        }
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("wpa_cli").args(["wps_pbc"]).output();
            if output.is_err() {
                debug!("WPS not supported. Ensure wpa_supplicant is running.");
                return Err(DriverError::execution("WPS not supported. Ensure wpa_supplicant is running."));
            }
        }
        #[cfg(target_os = "macos")]
        {
            debug!("WPS is not supported on macOS");
            return Err(DriverError::execution("WPS is not supported on macOS"));
        }
        info!("WPS connection initiated, timeout: {} seconds", timeout);
        return Ok(format!("WPS connection initiated. Please press the WPS button on your router within {} seconds.", timeout));
    }
}
