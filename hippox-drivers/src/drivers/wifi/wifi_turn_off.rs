//! WiFi turn off skill - disable WiFi adapter
use super::common::wifi_off;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for turning WiFi off
#[derive(Debug)]
pub struct WifiTurnOffDriver;
#[async_trait::async_trait]
impl Driver for WifiTurnOffDriver {
    fn name(&self) -> &str {
        return "wifi_turn_off";
    }
    fn description(&self) -> &str {
        return "Turn off the WiFi adapter/radio";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to disable WiFi. This will disconnect from any current network and turn off the radio.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_turn_off"
        }));
    }
    fn example_output(&self) -> String {
        return "WiFi turned off".to_string();
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
        debug!("Executing wifi_turn_off driver");
        wifi_off().map_err(|e| {
            debug!("Failed to turn WiFi off: {}", e);
            return DriverError::execution(format!("Failed to turn WiFi off: {}", e));
        })?;
        info!("WiFi turned off");
        return Ok("WiFi turned off".to_string());
    }
}
