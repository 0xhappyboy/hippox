//! Process termination by name driver
//!
//! This driver provides functionality to terminate all processes with a given name.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_process::common::kill_processes_by_name,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for terminating processes by name
#[derive(Debug)]
pub struct ProcessKillByNameDriver;
#[async_trait::async_trait]
impl Driver for ProcessKillByNameDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "process_kill_by_name"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Terminate all processes with a given name"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to stop all instances of an application"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "Process name to terminate (case-insensitive)".to_string(),
                required: true,
                default: None,
                example: Some(json!("chrome")),
                enum_values: None,
            },
            DriverParameter {
                name: "force".to_string(),
                param_type: "boolean".to_string(),
                description: "Force kill (SIGKILL instead of SIGTERM)".to_string(),
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
            "action": "process_kill_by_name",
            "parameters": {
                "name": "notepad.exe"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Terminated 3 process(es) matching 'notepad.exe'".to_string();
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
        debug!("Executing process_kill_by_name driver");
        let name = parameters.get("name").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("name"))?;
        let force = parameters.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
        info!("Terminating processes by name: '{}', force: {}", name, force);
        let killed = kill_processes_by_name(name, force).map_err(|e| DriverError::execution(e))?;
        info!("Terminated {} process(es) matching '{}'", killed, name);
        return Ok(format!("Terminated {} process(es) matching '{}'", killed, name));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_process_kill_by_name_skill() {
        let skill = ProcessKillByNameDriver;
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("nonexistent_process_xyz"));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("0"));
    }
}
