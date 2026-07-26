//! Application launch with arguments driver
//!
//! This driver provides functionality to launch an application with
//! specific command-line arguments.
use super::common::launch_app_with_args;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for launching applications with command-line arguments
#[derive(Debug)]
pub struct ApplicationControlLaunchWithArgsDriver;
#[async_trait::async_trait]
impl Driver for ApplicationControlLaunchWithArgsDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "application_control_launch_with_args"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Launch an application with command-line arguments"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to start an application with specific command-line arguments."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the application executable".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("notepad.exe".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "args".to_string(),
                param_type: "array".to_string(),
                description: "Array of command-line arguments".to_string(),
                required: true,
                default: None,
                example: Some(json!(["file.txt"])),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "application_control_launch_with_args",
            "parameters": {
                "path": "notepad.exe",
                "args": ["file.txt"]
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
        debug!("Executing application_control_launch_with_args driver");
        // Extract the application path parameter
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'path' parameter");
            DriverError::missing_parameter("path")
        })?;
        // Extract the arguments parameter
        let args = parameters.get("args").and_then(|v| v.as_array()).ok_or_else(|| {
            debug!("Missing 'args' parameter");
            DriverError::missing_parameter("args")
        })?;
        let args_str: Vec<String> = args.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
        debug!("Launching application with args: {} {:?}", path, args_str);
        // Launch the application with arguments
        let pid = launch_app_with_args(path, &args_str).map_err(|e| DriverError::execution(format!("Failed to launch with args: {}", e)))?;
        info!("Application launched with args: {} (PID: {})", path, pid);
        Ok(format!("Application launched with PID: {}", pid))
    }
}
