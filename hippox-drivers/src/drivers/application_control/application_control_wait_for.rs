//! Application wait for driver
//!
//! This driver provides functionality to wait for an application
//! to start running, with a configurable timeout.
use super::common::find_process_by_name;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info, warn};
/// Driver for waiting for applications to start
#[derive(Debug)]
pub struct ApplicationControlWaitForDriver;
#[async_trait::async_trait]
impl Driver for ApplicationControlWaitForDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "application_control_wait_for"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Wait for an application to start running"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to wait until an application has launched."
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
            "action": "application_control_wait_for",
            "parameters": {
                "name": "notepad.exe",
                "timeout_ms": 10000
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Application started within timeout".to_string()
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
        debug!("Executing application_control_wait_for driver");
        // Extract the application name parameter
        let name = parameters.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'name' parameter");
            DriverError::missing_parameter("name")
        })?;
        // Extract optional timeout parameter (default 30000ms)
        let timeout_ms = parameters.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(30000);
        debug!("Waiting for application '{}' (timeout: {}ms)", name, timeout_ms);
        let start = Instant::now();
        // Loop until application is found or timeout
        loop {
            if start.elapsed() > std::time::Duration::from_millis(timeout_ms) {
                let msg = format!("Timeout waiting for application '{}' to start", name);
                warn!("{}", msg);
                return Err(DriverError::Timeout { duration: Some(format!("{}ms", timeout_ms)) });
            }
            let processes = find_process_by_name(name).map_err(|e| DriverError::execution(format!("Failed to find process: {}", e)))?;
            if !processes.is_empty() {
                info!("Application '{}' started after {}ms", name, start.elapsed().as_millis());
                return Ok("Application started within timeout".to_string());
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }
}
