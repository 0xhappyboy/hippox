//! Service reverse dependencies Driver

use super::common::get_reverse_dependencies;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ServiceReverseDependenciesDriver;

#[async_trait::async_trait]
impl Driver for ServiceReverseDependenciesDriver {
    fn name(&self) -> &str {
        "service_reverse_dependencies"
    }

    fn description(&self) -> &str {
        "List services that depend on this service"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see which other services depend on this service."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("network.target".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_reverse_dependencies",
            "parameters": {
                "service_name": "network.target"
            }
        })
    }

    fn example_output(&self) -> String {
        "Services depending on network.target:\n1. ssh\n2. nginx".to_string()
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
        let deps = get_reverse_dependencies(service_name)?;
        if deps.is_empty() {
            return Ok(format!("No services depend on {}", service_name));
        }
        let mut result = format!("Services depending on {}:\n", service_name);
        for (i, dep) in deps.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, dep));
        }
        Ok(result)
    }
}
