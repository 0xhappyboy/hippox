//! WiFi hotspot stop skill - stop mobile hotspot

use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug)]
pub struct WifiHotspotStopDriver;

#[async_trait::async_trait]
impl Driver for WifiHotspotStopDriver {
    fn name(&self) -> &str {
        "wifi_hotspot_stop"
    }

    fn description(&self) -> &str {
        "Stop the mobile hotspot (soft AP mode)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to stop the WiFi hotspot that was previously created."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_hotspot_stop"
        })
    }

    fn example_output(&self) -> String {
        "Hotspot stopped".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Wifi
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        #[cfg(target_os = "windows")]
        {
            Command::new("netsh")
                .args(["wlan", "stop", "hostednetwork"])
                .output()?;
        }

        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("nmcli")
                .args(["connection", "down", "Hotspot"])
                .output();
        }

        Ok("Hotspot stopped".to_string())
    }
}
