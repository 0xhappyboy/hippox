//! WiFi turn on skill - enable WiFi adapter

use super::common::wifi_on;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct WifiTurnOnDriver;

#[async_trait::async_trait]
impl Driver for WifiTurnOnDriver {
    fn name(&self) -> &str {
        "wifi_turn_on"
    }

    fn description(&self) -> &str {
        "Turn on the WiFi adapter/radio"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to enable WiFi when it is turned off. After turning on, the device may automatically connect to known networks."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_turn_on"
        })
    }

    fn example_output(&self) -> String {
        "WiFi turned on".to_string()
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
        wifi_on()?;
        Ok("WiFi turned on".to_string())
    }
}
