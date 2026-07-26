//! OS set environment variable driver
//!
//! This driver provides functionality to set environment variables temporarily.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
use tracing::{debug, info};
/// Driver for setting environment variables
#[derive(Debug)]
pub struct OsSetEnvDriver;
#[async_trait::async_trait]
impl Driver for OsSetEnvDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_set_env"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set an environment variable (temporary, current process only)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to set a temporary environment variable for the current session"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
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
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_set_env",
            "parameters": {
                "name": "MY_VAR",
                "value": "my_value"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Environment variable MY_VAR set to my_value".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemBasis;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing os_set_env driver");
        let name = parameters.get("name").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("name"))?;
        let value = parameters.get("value").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("value"))?;
        unsafe {
            env::set_var(name, value);
        }
        info!("Environment variable {} set to {}", name, value);
        return Ok(format!("Environment variable {} set to {}", name, value));
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
