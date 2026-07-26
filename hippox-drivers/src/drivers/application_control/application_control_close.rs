//! Application close driver
//!
//! This driver provides functionality to gracefully close an application
//! by sending a close message to its main window.
use super::common::{close_process_window, find_process_by_name};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for closing applications gracefully
#[derive(Debug)]
pub struct ApplicationControlCloseDriver;
#[async_trait::async_trait]
impl Driver for ApplicationControlCloseDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "application_control_close"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Close an application gracefully"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to close an application by sending a close message to its window."
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
            "action": "application_control_close",
            "parameters": {
                "name": "notepad.exe"
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Application closed".to_string()
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
        debug!("Executing application_control_close driver");
        // Extract the application name parameter
        let name = parameters.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'name' parameter");
            DriverError::missing_parameter("name")
        })?;
        debug!("Searching for processes matching: {}", name);
        // Find all processes matching the name
        let processes = find_process_by_name(name).map_err(|e| {
            let msg = format!("Failed to find process: {}", e);
            warn!("{}", msg);
            DriverError::execution(msg)
        })?;
        // Check if any processes were found
        if processes.is_empty() {
            let msg = format!("No process found with name: {}", name);
            warn!("{}", msg);
            return Err(DriverError::execution(msg));
        }
        info!("Found {} processes matching '{}'", processes.len(), name);
        // Send close message to each matching process
        for process in processes {
            debug!("Closing process: {} (PID: {})", process.name, process.pid);
            let _ = close_process_window(process.pid);
        }
        info!("Application close messages sent for: {}", name);
        Ok("Application closed".to_string())
    }
}
