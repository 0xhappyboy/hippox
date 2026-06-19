//! Service import Driver

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::import_service_config;
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct ServiceImportDriver;

#[async_trait::async_trait]
impl Driver for ServiceImportDriver {
    fn name(&self) -> &str {
        "service_import"
    }

    fn description(&self) -> &str {
        "Import service configuration from file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to restore a service's configuration from a backup."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "service_name".to_string(),
                param_type: "string".to_string(),
                description: "Name of the service".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("nginx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "input_path".to_string(),
                param_type: "string".to_string(),
                description: "Path to import configuration from".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/nginx.service.backup".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_import",
            "parameters": {
                "service_name": "nginx",
                "input_path": "/tmp/nginx.service.backup"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx configuration imported from /tmp/nginx.service.backup".to_string()
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
        let input_path = parameters
            .get("input_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'input_path' parameter"))?;
        import_service_config(service_name, input_path)?;
        Ok(format!(
            "Service {} configuration imported from {}",
            service_name, input_path
        ))
    }
}
