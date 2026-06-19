//! Display refresh rate get skill

use super::common::get_refresh_rate;
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
pub struct DisplayControlRefreshRateGetDriver;

#[async_trait::async_trait]
impl Driver for DisplayControlRefreshRateGetDriver {
    fn name(&self) -> &str {
        "display_control_refresh_rate_get"
    }

    fn description(&self) -> &str {
        "Get the current display refresh rate in Hz"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the monitor's refresh rate (e.g., 60Hz, 144Hz)."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_refresh_rate_get"
        })
    }

    fn example_output(&self) -> String {
        "Display refresh rate: 60 Hz".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Display
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let rate = get_refresh_rate(None)?;

        Ok(format!("Display refresh rate: {} Hz", rate))
    }
}
