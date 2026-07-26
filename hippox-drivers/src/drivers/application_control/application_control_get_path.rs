//! Application get path driver
//!
//! This driver provides functionality to get the full path of an application
//! executable by searching common system locations.
use super::common::get_app_path;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for getting application executable paths
#[derive(Debug)]
pub struct ApplicationControlGetPathDriver;
#[async_trait::async_trait]
impl Driver for ApplicationControlGetPathDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "application_control_get_path"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the full path of an application executable"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to find where an application is installed."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "name".to_string(),
            param_type: "string".to_string(),
            description: "Application name (e.g., 'notepad', 'chrome')".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("notepad".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "application_control_get_path",
            "parameters": {
                "name": "notepad"
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Application path: C:\\Windows\\System32\\notepad.exe".to_string()
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        DriverCategory::Application
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing application_control_get_path driver");
        // Extract the application name parameter
        let name = parameters.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'name' parameter");
            DriverError::missing_parameter("name")
        })?;
        debug!("Getting application path for: {}", name);
        // Get the application path
        let path = get_app_path(name).map_err(|e| {
            let msg = format!("Failed to get application path: {}", e);
            warn!("{}", msg);
            DriverError::execution(msg)
        })?;
        info!("Application path found: {} -> {}", name, path);
        Ok(format!("Application path: {}", path))
    }
}
