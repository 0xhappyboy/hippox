//! Display primary get skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::DriverCallback;
use crate::DriverContext;
use super::common::get_primary_display;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct DisplayControlPrimaryGetDriver;

#[async_trait::async_trait]
impl Driver for DisplayControlPrimaryGetDriver {
    fn name(&self) -> &str {
        "display_control_primary_get"
    }

    fn description(&self) -> &str {
        "Get information about the primary display"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get details about the main monitor."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_primary_get"
        })
    }

    fn example_output(&self) -> String {
        "Primary display: Primary Display (1920x1080, 60Hz)".to_string()
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
        let display = get_primary_display()?;

        Ok(format!(
            "Primary display: {} ({}x{} @ {}Hz)",
            display.name, display.width, display.height, display.refresh_rate
        ))
    }
}
