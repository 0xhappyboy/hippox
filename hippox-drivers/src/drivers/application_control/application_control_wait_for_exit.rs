//! Application wait for exit driver
//!
//! This driver provides functionality to wait for an application
//! to exit, with a configurable timeout.
use super::common::{find_process_by_name, wait_for_exit};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for waiting for applications to exit
#[derive(Debug)]
pub struct ApplicationControlWaitForExitDriver;
#[async_trait::async_trait]
impl Driver for ApplicationControlWaitForExitDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "application_control_wait_for_exit"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Wait for an application to exit"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to wait until an application has completely closed."
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
                name: "timeout_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum wait time in milliseconds".to_string(),
                required: false,
                default: Some(Value::Number(30000.into())),
                example: Some(Value::Number(10000.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "application_control_wait_for_exit",
            "parameters": {
                "name": "notepad.exe",
                "timeout_ms": 10000
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Application exited within timeout".to_string()
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
        debug!("Executing application_control_wait_for_exit driver");
        // Extract the application name parameter
        let name = parameters.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'name' parameter");
            DriverError::missing_parameter("name")
        })?;
        // Extract optional timeout parameter (default 30000ms)
        let timeout_ms = parameters.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(30000);
        debug!("Waiting for application '{}' to exit (timeout: {}ms)", name, timeout_ms);
        // Find the process
        let processes = find_process_by_name(name).map_err(|e| DriverError::execution(format!("Failed to find process: {}", e)))?;
        // If process is not running, return immediately
        if processes.is_empty() {
            info!("Application '{}' is not running", name);
            return Ok("Application is not running".to_string());
        }
        let pid = processes[0].pid;
        debug!("Waiting for process {} to exit", pid);
        // Wait for the process to exit
        let exited = wait_for_exit(pid, timeout_ms).await.map_err(|e| DriverError::execution(format!("Failed to wait for exit: {}", e)))?;
        if exited {
            info!("Application '{}' exited within timeout", name);
            Ok("Application exited within timeout".to_string())
        } else {
            let msg = format!("Timeout waiting for application '{}' to exit", name);
            warn!("{}", msg);
            Err(DriverError::Timeout { duration: Some(format!("{}ms", timeout_ms)) })
        }
    }
}
