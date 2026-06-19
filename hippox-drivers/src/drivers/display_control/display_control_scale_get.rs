//! Display scale get skill

use super::common::get_scale;
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
pub struct DisplayControlScaleGetDriver;

#[async_trait::async_trait]
impl Driver for DisplayControlScaleGetDriver {
    fn name(&self) -> &str {
        "display_control_scale_get"
    }

    fn description(&self) -> &str {
        "Get the display scaling factor"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the DPI scaling factor (e.g., 1.0 for 100%, 1.5 for 150%)."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_scale_get"
        })
    }

    fn example_output(&self) -> String {
        "Display scale: 1.5x".to_string()
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
        let scale = get_scale(None)?;

        Ok(format!("Display scale: {:.1}x", scale))
    }
}
