//! WiFi hotspot stop skill - stop mobile hotspot
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for stopping a mobile hotspot
#[derive(Debug)]
pub struct WifiHotspotStopDriver;
#[async_trait::async_trait]
impl Driver for WifiHotspotStopDriver {
    fn name(&self) -> &str {
        return "wifi_hotspot_stop";
    }
    fn description(&self) -> &str {
        return "Stop the mobile hotspot (soft AP mode)";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to stop the WiFi hotspot that was previously created.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_hotspot_stop"
        }));
    }
    fn example_output(&self) -> String {
        return "Hotspot stopped".to_string();
    }
    fn category(&self) -> DriverCategory {
        return DriverCategory::Wifi;
    }
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing wifi_hotspot_stop driver");
        #[cfg(target_os = "windows")]
        {
            Command::new("netsh").args(["wlan", "stop", "hostednetwork"]).output().map_err(|e| {
                debug!("Failed to stop hostednetwork: {}", e);
                return DriverError::execution(format!("Failed to stop hostednetwork: {}", e));
            })?;
        }
        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("nmcli").args(["connection", "down", "Hotspot"]).output();
        }
        info!("Hotspot stopped");
        return Ok("Hotspot stopped".to_string());
    }
}
