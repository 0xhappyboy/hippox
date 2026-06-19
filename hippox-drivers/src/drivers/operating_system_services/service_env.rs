//! Service environment variables Driver

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_service_env;
use crate::{
    DriverCallback, DriverCategory, DriverContext, types::{Driver, DriverParameter}
};

#[derive(Debug)]
pub struct ServiceEnvDriver;

#[async_trait::async_trait]
impl Driver for ServiceEnvDriver {
    fn name(&self) -> &str {
        "service_env"
    }

    fn description(&self) -> &str {
        "View service environment variables"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see the environment variables available to a service."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_env",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx environment variables:\nPATH=/usr/local/bin:/usr/bin\nUSER=www-data"
            .to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemServices
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let service_name = parameters
            .get("service_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'service_name' parameter"))?;
        let env = get_service_env(service_name)?;
        if env.is_empty() {
            return Ok(format!(
                "No environment variables found for service {}",
                service_name
            ));
        }
        let mut result = format!("Service {} environment variables:\n", service_name);
        for (key, value) in env {
            result.push_str(&format!("{}={}\n", key, value));
        }
        Ok(result)
    }
}
