//! Process termination by PID driver
//!
//! This driver provides functionality to terminate a process by its PID.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_process::common::{get_process_by_pid, kill_process},
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for terminating a process by PID
#[derive(Debug)]
pub struct ProcessKillDriver;
#[async_trait::async_trait]
impl Driver for ProcessKillDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "process_kill"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Terminate a process by its PID"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to stop a misbehaving or unwanted process"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "pid".to_string(),
                param_type: "integer".to_string(),
                description: "Process ID to terminate".to_string(),
                required: true,
                default: None,
                example: Some(json!(1234)),
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
            "action": "process_kill",
            "parameters": {
                "pid": 1234
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Process 1234 terminated successfully".to_string();
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
        debug!("Executing process_kill driver");
        let pid = parameters.get("pid").and_then(|v| v.as_u64()).ok_or_else(|| DriverError::missing_parameter("pid"))? as u32;
        let force = parameters.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
        info!("Terminating process PID: {}, force: {}", pid, force);
        // Check if process exists
        if get_process_by_pid(pid).is_none() {
            debug!("Process with PID {} not found", pid);
            return Err(DriverError::execution(format!("Process with PID {} not found", pid)));
        }
        kill_process(pid, force).map_err(|e| DriverError::execution(e))?;
        info!("Process {} terminated successfully", pid);
        return Ok(format!("Process {} terminated successfully", pid));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_process_kill_invalid_pid() {
        let skill = ProcessKillDriver;
        let mut params = HashMap::new();
        params.insert("pid".to_string(), json!(99999999));
        let result = skill.execute(&params, None, None).await;
        assert!(result.is_err());
    }
}
