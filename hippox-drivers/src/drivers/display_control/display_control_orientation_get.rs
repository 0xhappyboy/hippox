//! Display orientation get skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::DriverCallback;
use crate::DriverContext;
use super::common::get_orientation;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct DisplayControlOrientationGetDriver;

#[async_trait::async_trait]
impl Driver for DisplayControlOrientationGetDriver {
    fn name(&self) -> &str {
        "display_control_orientation_get"
    }

    fn description(&self) -> &str {
        "Get the display orientation (landscape, portrait, etc.)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check if the display is in landscape or portrait mode."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_orientation_get"
        })
    }

    fn example_output(&self) -> String {
        "Display orientation: landscape".to_string()
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
        let orientation = get_orientation(None)?;

        Ok(format!("Display orientation: {}", orientation))
    }
}
