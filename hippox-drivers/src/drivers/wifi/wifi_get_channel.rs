//! WiFi get channel skill - get current WiFi channel and frequency
use super::common::get_wifi_status;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting current WiFi channel
#[derive(Debug)]
pub struct WifiGetChannelDriver;
#[async_trait::async_trait]
impl Driver for WifiGetChannelDriver {
    fn name(&self) -> &str {
        return "wifi_get_channel";
    }
    fn description(&self) -> &str {
        return "Get the current WiFi channel and frequency band (2.4GHz/5GHz/6GHz)";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see what channel your WiFi is using. Useful for diagnosing interference issues.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_get_channel"
        }));
    }
    fn example_output(&self) -> String {
        return "Current channel: 6 (2.4GHz)".to_string();
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
        debug!("Executing wifi_get_channel driver");
        let status = get_wifi_status().map_err(|e| {
            debug!("Failed to get WiFi status: {}", e);
            return DriverError::execution(format!("Failed to get WiFi status: {}", e));
        })?;
        if !status.connected {
            info!("Not connected to WiFi");
            return Ok("Not connected to WiFi".to_string());
        }
        if let Some(channel) = status.channel {
            let freq = if channel <= 14 {
                "2.4GHz"
            } else if channel <= 64 {
                "5GHz"
            } else {
                "6GHz"
            };
            info!("Current channel: {} ({})", channel, freq);
            return Ok(format!("Current channel: {} ({})", channel, freq));
        } else {
            info!("Channel information not available");
            return Ok("Channel information not available".to_string());
        }
    }
}
