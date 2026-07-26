//! OS get environment variable driver
//!
//! This driver provides functionality to get environment variables.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
use tracing::{debug, info};
/// Driver for getting environment variables
#[derive(Debug)]
pub struct OsGetEnvDriver;
#[async_trait::async_trait]
impl Driver for OsGetEnvDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_get_env"
    }
    /// Returns a brief description of the driver's functionality   
    fn description(&self) -> &str {
        "Get environment variables"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the value of an environment variable, or list all variables"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "name".to_string(),
            param_type: "string".to_string(),
            description: "Environment variable name (optional, returns all if not specified)".to_string(),
            required: false,
            default: None,
            example: Some(Value::String("PATH".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_get_env",
            "parameters": {
                "name": "PATH"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "PATH=/usr/local/bin:/usr/bin:/bin".to_string();
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
        debug!("Executing os_get_env driver");
        let name = parameters.get("name").and_then(|v| v.as_str());
        if let Some(name) = name {
            match env::var(name) {
                Ok(value) => {
                    info!("Environment variable retrieved: {}", name);
                    return Ok(format!("{}={}", name, value));
                }
                Err(_) => {
                    info!("Environment variable not found: {}", name);
                    return Ok(format!("Environment variable '{}' not found", name));
                }
            }
        } else {
            debug!("Listing all environment variables");
            let mut result = String::from("Environment variables:\n");
            let mut vars: Vec<(String, String)> = env::vars().collect();
            vars.sort_by(|a, b| a.0.cmp(&b.0));
            for (key, value) in vars.iter().take(100) {
                let display_value = if value.len() > 200 { format!("{}...", &value[..200]) } else { value.clone() };
                result.push_str(&format!("  {}={}\n", key, display_value));
            }
            if vars.len() > 100 {
                result.push_str(&format!("  ... and {} more\n", vars.len() - 100));
            }
            info!("Listed {} environment variables", vars.len().min(100));
            return Ok(result);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_get_env_metadata() {
        let driver = OsGetEnvDriver;
        assert_eq!(driver.name(), "os_get_env");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
