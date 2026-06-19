//! Service copy Driver

use crate::{DriverCallback, DriverContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::copy_service;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct ServiceCopyDriver;

#[async_trait::async_trait]
impl Driver for ServiceCopyDriver {
    fn name(&self) -> &str {
        "service_copy"
    }

    fn description(&self) -> &str {
        "Copy a service configuration to create a new service"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to duplicate an existing service configuration."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "source_service".to_string(),
                param_type: "string".to_string(),
                description: "Name of the service to copy from".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("nginx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "dest_service".to_string(),
                param_type: "string".to_string(),
                description: "Name of the new service".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("myapp".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_copy",
            "parameters": {
                "source_service": "nginx",
                "dest_service": "myapp"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service myapp copied from nginx".to_string()
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
        let source = parameters
            .get("source_service")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source_service' parameter"))?;
        let dest = parameters
            .get("dest_service")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'dest_service' parameter"))?;
        copy_service(source, dest)?;
        Ok(format!("Service {} copied from {}", dest, source))
    }
}
