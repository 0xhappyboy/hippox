//! WiFi turn off skill - disable WiFi adapter

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::wifi_off;
use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};

#[derive(Debug)]
pub struct WifiTurnOffDriver;

#[async_trait::async_trait]
impl Driver for WifiTurnOffDriver {
    fn name(&self) -> &str {
        "wifi_turn_off"
    }

    fn description(&self) -> &str {
        "Turn off the WiFi adapter/radio"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to disable WiFi. This will disconnect from any current network and turn off the radio."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "wifi_turn_off"
        })
    }

    fn example_output(&self) -> String {
        "WiFi turned off".to_string()
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
        wifi_off()?;
        Ok("WiFi turned off".to_string())
    }
}
