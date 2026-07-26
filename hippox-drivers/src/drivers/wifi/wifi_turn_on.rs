//! WiFi turn on skill - enable WiFi adapter
use super::common::wifi_on;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for turning WiFi on
#[derive(Debug)]
pub struct WifiTurnOnDriver;
#[async_trait::async_trait]
impl Driver for WifiTurnOnDriver {
    fn name(&self) -> &str {
        return "wifi_turn_on";
    }
    fn description(&self) -> &str {
        return "Turn on the WiFi adapter/radio";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to enable WiFi when it is turned off. After turning on, the device may automatically connect to known networks.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "wifi_turn_on"
        }));
    }
    fn example_output(&self) -> String {
        return "WiFi turned on".to_string();
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
        debug!("Executing wifi_turn_on driver");
        wifi_on().map_err(|e| {
            debug!("Failed to turn WiFi on: {}", e);
            return DriverError::execution(format!("Failed to turn WiFi on: {}", e));
        })?;
        info!("WiFi turned on");
        return Ok("WiFi turned on".to_string());
    }
}
