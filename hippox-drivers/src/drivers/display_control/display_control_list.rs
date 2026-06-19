//! Display list skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::DriverCallback;
use crate::DriverContext;
use super::common::list_displays;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct DisplayControlListDriver;

#[async_trait::async_trait]
impl Driver for DisplayControlListDriver {
    fn name(&self) -> &str {
        "display_control_list"
    }

    fn description(&self) -> &str {
        "List all connected displays/monitors"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get information about all connected monitors."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_list"
        })
    }

    fn example_output(&self) -> String {
        "Found 2 displays:\n1. Primary Display (1920x1080, 60Hz)\n2. Secondary Display (1920x1080, 60Hz)".to_string()
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
        let displays = list_displays()?;

        if displays.is_empty() {
            return Ok("No displays found".to_string());
        }

        let mut result = format!("Found {} displays:\n", displays.len());
        for (i, display) in displays.iter().enumerate() {
            let primary_marker = if display.is_primary { " (primary)" } else { "" };
            result.push_str(&format!(
                "{}. {}{} - {}x{} @ {}Hz, scale: {:.1}x\n",
                i + 1,
                display.name,
                primary_marker,
                display.width,
                display.height,
                display.refresh_rate,
                display.scale
            ));
        }

        Ok(result)
    }
}
