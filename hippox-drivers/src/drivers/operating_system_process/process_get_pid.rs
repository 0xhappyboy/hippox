//! Process PID retrieval driver
//!
//! This driver provides functionality to get the PID(s) of a process by name.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_process::common::get_pids_by_name,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting the PID of a process by name
#[derive(Debug)]
pub struct ProcessGetPidDriver;
#[async_trait::async_trait]
impl Driver for ProcessGetPidDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "process_get_pid"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the PID(s) of a process by name"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need the process ID of an application for monitoring or management"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "Process name to find".to_string(),
                required: true,
                default: None,
                example: Some(json!("python")),
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
            DriverParameter {
                name: "first_only".to_string(),
                param_type: "boolean".to_string(),
                description: "Return only the first PID found (default: false)".to_string(),
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
            "action": "process_get_pid",
            "parameters": {
                "name": "sshd"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Found PIDs: 1234, 5678".to_string();
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
        debug!("Executing process_get_pid driver");
        let name = parameters.get("name").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("name"))?;
        let exact_match = parameters.get("exact_match").and_then(|v| v.as_bool()).unwrap_or(false);
        let first_only = parameters.get("first_only").and_then(|v| v.as_bool()).unwrap_or(false);
        info!("Getting PIDs for process: '{}', exact_match: {}", name, exact_match);
        let pids = get_pids_by_name(name, exact_match);
        if pids.is_empty() {
            info!("No process found matching '{}'", name);
            return Ok(format!("No process found matching '{}'", name));
        } else if first_only {
            info!("Returning first PID: {}", pids[0]);
            return Ok(format!("PID: {}", pids[0]));
        } else {
            info!("Found {} PIDs for '{}'", pids.len(), name);
            return Ok(format!("Found PIDs: {}", pids.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ")));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_process_get_pid_skill() {
        let skill = ProcessGetPidDriver;
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("system"));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_process_get_pid_first_only() {
        let skill = ProcessGetPidDriver;
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("system"));
        params.insert("first_only".to_string(), json!(true));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("PID:"));
    }
}
