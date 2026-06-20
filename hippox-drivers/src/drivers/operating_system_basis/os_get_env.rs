//! OS get environment variable driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
#[derive(Debug)]
pub struct OsGetEnvDriver;
#[async_trait::async_trait]
impl Driver for OsGetEnvDriver {
    fn name(&self) -> &str {
        "os_get_env"
    }
    fn description(&self) -> &str {
        "Get environment variables"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to get the value of an environment variable, or list all variables"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "name".to_string(),
            param_type: "string".to_string(),
            description: "Environment variable name (optional, returns all if not specified)"
                .to_string(),
            required: false,
            default: None,
            example: Some(Value::String("PATH".to_string())),
            enum_values: None,
        }]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_get_env",
            "parameters": {
                "name": "PATH"
            }
        })
    }
    fn example_output(&self) -> String {
        "PATH=/usr/local/bin:/usr/bin:/bin".to_string()
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
        let name = parameters.get("name").and_then(|v| v.as_str());
        if let Some(name) = name {
            match env::var(name) {
                Ok(value) => Ok(format!("{}={}", name, value)),
                Err(_) => Ok(format!("Environment variable '{}' not found", name)),
            }
        } else {
            let mut result = String::from("Environment variables:\n");
            let mut vars: Vec<(String, String)> = env::vars().collect();
            vars.sort_by(|a, b| a.0.cmp(&b.0));
            for (key, value) in vars.iter().take(100) {
                let display_value = if value.len() > 200 {
                    format!("{}...", &value[..200])
                } else {
                    value.clone()
                };
                result.push_str(&format!("  {}={}\n", key, display_value));
            }
            if vars.len() > 100 {
                result.push_str(&format!("  ... and {} more\n", vars.len() - 100));
            }
            Ok(result)
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
