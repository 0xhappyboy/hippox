//! Process termination by name skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    DriverCallback, DriverCategory, DriverContext, operating_system_process::common::kill_processes_by_name, types::{Driver, DriverParameter}
};

/// Driver for terminating processes by name
#[derive(Debug)]
pub struct ProcessKillByNameDriver;

#[async_trait::async_trait]
impl Driver for ProcessKillByNameDriver {
    fn name(&self) -> &str {
        "process_kill_by_name"
    }

    fn description(&self) -> &str {
        "Terminate all processes with a given name"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to stop all instances of an application"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "process_kill_by_name",
            "parameters": {
                "name": "notepad.exe"
            }
        })
    }

    fn example_output(&self) -> String {
        "Terminated 3 process(es) matching 'notepad.exe'".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemProcess
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let name = parameters
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;

        let force = parameters
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let killed = kill_processes_by_name(name, force).map_err(|e| anyhow::anyhow!(e))?;

        Ok(format!(
            "Terminated {} process(es) matching '{}'",
            killed, name
        ))
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
