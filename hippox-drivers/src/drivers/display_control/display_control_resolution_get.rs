//! Display resolution get skill

use super::common::get_resolution;
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
pub struct DisplayControlResolutionGetDriver;

#[async_trait::async_trait]
impl Driver for DisplayControlResolutionGetDriver {
    fn name(&self) -> &str {
        "display_control_resolution_get"
    }

    fn description(&self) -> &str {
        "Get the current display resolution"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the width and height of the primary display."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_resolution_get"
        })
    }

    fn example_output(&self) -> String {
        "Current resolution: 1920x1080".to_string()
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
        let (width, height) = get_resolution(None)?;

        Ok(format!("Current resolution: {}x{}", width, height))
    }
}
