//! Application restart driver
//!
//! This driver provides functionality to restart an application by
//! closing it and then relaunching it.

use super::common::{close_process_window, find_process_by_name, launch_app, wait_for_exit};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};

/// Driver for restarting applications
#[derive(Debug)]
pub struct ApplicationControlRestartDriver;

#[async_trait::async_trait]
impl Driver for ApplicationControlRestartDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "application_control_restart"
    }

    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Restart an application (close and relaunch)"
    }

    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to restart a hung or misbehaving application."
    }

    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "Application name or process name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("notepad.exe".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the application executable (if different from name)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("C:\\Windows\\System32\\notepad.exe".to_string())),
                enum_values: None,
            },
        ];
    }

    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "application_control_restart",
            "parameters": {
                "name": "notepad.exe"
            }
        }))
    }

    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Application restarted".to_string()
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
        debug!("Executing application_control_restart driver");

        // Extract the application name parameter
        let name = parameters.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'name' parameter");
            DriverError::missing_parameter("name")
        })?;

        // Extract optional path parameter (fallback to name)
        let path = parameters.get("path").and_then(|v| v.as_str()).unwrap_or(name);

        debug!("Restarting application: {} (path: {})", name, path);

        // Find and close existing instances
        let processes = find_process_by_name(name).map_err(|e| DriverError::execution(format!("Failed to find process: {}", e)))?;

        if !processes.is_empty() {
            info!("Found {} instances of '{}' to close", processes.len(), name);

            for process in processes {
                debug!("Closing process: {} (PID: {})", process.name, process.pid);
                let _ = close_process_window(process.pid);
                let _ = wait_for_exit(process.pid, 5000).await;
            }
        } else {
            debug!("No running instances found for: {}", name);
        }

        // Launch new instance
        debug!("Launching new instance: {}", path);
        let pid = launch_app(path).map_err(|e| DriverError::execution(format!("Failed to launch application: {}", e)))?;

        info!("Application restarted: {} (PID: {})", path, pid);
        Ok(format!("Application restarted with PID: {}", pid))
    }
}
