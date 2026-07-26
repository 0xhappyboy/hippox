//! Display list skill
//!
//! This driver provides functionality to list all connected displays
//! and their properties on the system.
use super::common::list_displays;
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing all connected displays
#[derive(Debug)]
pub struct DisplayControlListDriver;
#[async_trait::async_trait]
impl Driver for DisplayControlListDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "display_control_list"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "List all connected displays/monitors"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get information about all connected monitors."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "display_control_list"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Found 2 displays:\n1. Primary Display (1920x1080, 60Hz)\n2. Secondary Display (1920x1080, 60Hz)".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Display;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing display_control_list driver");
        let displays = list_displays().map_err(|e| {
            debug!("Failed to list displays: {}", e);
            return crate::DriverError::execution(format!("Failed to list displays: {}", e));
        })?;
        if displays.is_empty() {
            info!("No displays found");
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
        info!("Listed {} displays", displays.len());
        return Ok(result);
    }
}
