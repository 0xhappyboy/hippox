//! OS set environment variable driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
#[derive(Debug)]
pub struct OsSetEnvDriver;
#[async_trait::async_trait]
impl Driver for OsSetEnvDriver {
    fn name(&self) -> &str {
        "os_set_env"
    }
    fn description(&self) -> &str {
        "Set an environment variable (temporary, current process only)"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to set a temporary environment variable for the current session"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "name".to_string(),
                param_type: "string".to_string(),
                description: "Environment variable name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("MY_VAR".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "value".to_string(),
                param_type: "string".to_string(),
                description: "Value to set".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("my_value".to_string())),
                enum_values: None,
            },
        ]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_set_env",
            "parameters": {
                "name": "MY_VAR",
                "value": "my_value"
            }
        })
    }
    fn example_output(&self) -> String {
        "Environment variable MY_VAR set to my_value".to_string()
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemBasis
    }
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let name = parameters
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;
        let value = parameters
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'value' parameter"))?;
        unsafe {
            env::set_var(name, value);
        }
        Ok(format!("Environment variable {} set to {}", name, value))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_set_env_metadata() {
        let driver = OsSetEnvDriver;
        assert_eq!(driver.name(), "os_set_env");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
