//! WiFi band preference get skill - get current frequency band preference
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for getting WiFi frequency band preference
#[derive(Debug)]
pub struct WifiBandPreferenceGetDriver;
#[async_trait::async_trait]
impl Driver for WifiBandPreferenceGetDriver {
    fn name(&self) -> &str {
        return "wifi_band_preference_get";
    }
    fn description(&self) -> &str {
        return "Get the current WiFi frequency band preference setting";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see which band (2.4GHz/5GHz/6GHz) is currently preferred.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_band_preference_get"
        }));
    }
    fn example_output(&self) -> String {
        return "Current band preference: 5GHz".to_string();
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
        debug!("Executing wifi_band_preference_get driver");
        let result = String::new();
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("netsh").args(["wlan", "show", "settings"]).output().map_err(|e| {
                debug!("Failed to get WiFi settings: {}", e);
                return DriverError::execution(format!("Failed to get WiFi settings: {}", e));
            })?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("Band") || line.contains("频段") {
                    if let Some(band) = line.split(':').nth(1) {
                        let result = format!("Current band preference: {}", band.trim());
                        info!("Band preference retrieved: {}", band.trim());
                        return Ok(result);
                    }
                }
            }
        }
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("iw").args(["reg", "get"]).output().map_err(|e| {
                debug!("Failed to get regulatory info: {}", e);
                return DriverError::execution(format!("Failed to get regulatory info: {}", e));
            })?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("2.4") && stdout.contains("5") {
                info!("Band preference: Auto (2.4GHz and 5GHz)");
                return Ok("Current band preference: Auto (2.4GHz and 5GHz)".to_string());
            } else if stdout.contains("2.4") {
                info!("Band preference: 2.4GHz");
                return Ok("Current band preference: 2.4GHz".to_string());
            } else if stdout.contains("5") {
                info!("Band preference: 5GHz");
                return Ok("Current band preference: 5GHz".to_string());
            }
        }
        info!("Band preference: Auto");
        return Ok("Current band preference: Auto".to_string());
    }
}
