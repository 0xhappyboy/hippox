//! Application launch driver
//!
//! This driver provides functionality to launch an application
//! by its path or name.
use super::common::launch_app;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for launching applications
#[derive(Debug)]
pub struct ApplicationControlLaunchDriver;
#[async_trait::async_trait]
impl Driver for ApplicationControlLaunchDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "application_control_launch"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Launch an application"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to start an application by its path or name."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the application executable".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("notepad.exe".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "application_control_launch",
            "parameters": {
                "path": "notepad.exe"
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Application launched with PID: 12345".to_string()
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
        debug!("Executing application_control_launch driver");
        // Extract the application path parameter
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'path' parameter");
            DriverError::missing_parameter("path")
        })?;
        debug!("Launching application: {}", path);
        // Launch the application
        let pid = launch_app(path).map_err(|e| DriverError::execution(format!("Failed to launch application: {}", e)))?;
        info!("Application launched: {} (PID: {})", path, pid);
        Ok(format!("Application launched with PID: {}", pid))
    }
}
