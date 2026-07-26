//! Process running check driver
//!
//! This driver provides functionality to check if a process with the given name is running.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_process::common::is_process_running,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for checking if a process is running
#[derive(Debug)]
pub struct ProcessIsRunningDriver;
#[async_trait::async_trait]
impl Driver for ProcessIsRunningDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "process_is_running"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Check if a process with the given name is running"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to verify if a service or application is currently running"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "Process name to check".to_string(),
                required: true,
                default: None,
                example: Some(json!("nginx")),
                enum_values: None,
            },
            DriverParameter {
                name: "exact_match".to_string(),
                param_type: "boolean".to_string(),
                description: "Require exact name match (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "process_is_running",
            "parameters": {
                "name": "docker"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Process 'docker' is running".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemProcess;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing process_is_running driver");
        let name = parameters.get("name").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("name"))?;
        let exact_match = parameters.get("exact_match").and_then(|v| v.as_bool()).unwrap_or(false);
        info!("Checking if process '{}' is running (exact: {})", name, exact_match);
        let running = is_process_running(name, exact_match);
        if running {
            info!("Process '{}' is running", name);
            return Ok(format!("Process '{}' is running", name));
        } else {
            info!("Process '{}' is not running", name);
            return Ok(format!("Process '{}' is not running", name));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_process_is_running_skill() {
        let skill = ProcessIsRunningDriver;
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("system"));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_ok());
    }
}
