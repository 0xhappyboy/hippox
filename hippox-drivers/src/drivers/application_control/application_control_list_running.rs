//! Application list running driver
//!
//! This driver provides functionality to list all currently running
//! applications and processes on the system.
use super::common::list_running_processes;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing running applications
#[derive(Debug)]
pub struct ApplicationControlListRunningDriver;
#[async_trait::async_trait]
impl Driver for ApplicationControlListRunningDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "application_control_list_running"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "List all currently running applications/processes"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to see what applications are currently active on the system."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "filter".to_string(),
                param_type: "string".to_string(),
                description: "Optional filter to narrow results".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("chrome".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of results to return".to_string(),
                required: false,
                default: Some(Value::Number(50.into())),
                example: Some(Value::Number(20.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "application_control_list_running"
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Found 127 processes:\n1. System (PID: 4)\n2. notepad.exe (PID: 12345)\n...".to_string()
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
        debug!("Executing application_control_list_running driver");
        // Extract optional filter parameter
        let filter = parameters.get("filter").and_then(|v| v.as_str());
        // Extract optional limit parameter (default 50)
        let limit = parameters.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
        debug!("Listing processes (filter: {:?}, limit: {})", filter, limit);
        // Get all running processes
        let mut processes = list_running_processes().map_err(|e| DriverError::execution(format!("Failed to list processes: {}", e)))?;
        // Apply filter if provided
        if let Some(f) = filter {
            let filter_lower = f.to_lowercase();
            processes.retain(|p| p.name.to_lowercase().contains(&filter_lower));
            debug!("Filtered to {} processes matching '{}'", processes.len(), f);
        }
        // Apply limit
        processes.truncate(limit);
        if processes.is_empty() {
            info!("No processes found");
            return Ok("No processes found".to_string());
        }
        // Build the result string
        let mut result = format!("Found {} processes:\n", processes.len());
        for (i, proc) in processes.iter().enumerate() {
            result.push_str(&format!("{}. {} (PID: {})\n", i + 1, proc.name, proc.pid));
        }
        info!("Listed {} processes", processes.len());
        Ok(result)
    }
}
