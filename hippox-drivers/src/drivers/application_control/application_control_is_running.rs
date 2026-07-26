//! Application is running check driver
//!
//! This driver provides functionality to check if an application is
//! currently running on the system.
use super::common::find_process_by_name;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for checking if an application is running
#[derive(Debug)]
pub struct ApplicationControlIsRunningDriver;
#[async_trait::async_trait]
impl Driver for ApplicationControlIsRunningDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "application_control_is_running"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Check if an application is currently running"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to check if a specific application is active."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "name".to_string(),
            param_type: "string".to_string(),
            description: "Application name or process name".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("notepad.exe".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "application_control_is_running",
            "parameters": {
                "name": "notepad.exe"
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Application is running: true".to_string()
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
        debug!("Executing application_control_is_running driver");
        // Extract the application name parameter
        let name = parameters.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'name' parameter");
            DriverError::missing_parameter("name")
        })?;
        debug!("Checking if application is running: {}", name);
        // Find processes matching the name
        let processes = find_process_by_name(name).map_err(|e| DriverError::execution(format!("Failed to find process: {}", e)))?;
        let is_running = !processes.is_empty();
        info!("Application '{}' running: {}", name, is_running);
        Ok(format!("Application is running: {}", is_running))
    }
}
