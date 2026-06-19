//! Service rename Driver

use super::common::rename_service;
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
pub struct ServiceRenameDriver;

#[async_trait::async_trait]
impl Driver for ServiceRenameDriver {
    fn name(&self) -> &str {
        "service_rename"
    }

    fn description(&self) -> &str {
        "Rename an existing service"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to rename a service."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "old_name".to_string(),
                param_type: "string".to_string(),
                description: "Current service name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("nginx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "new_name".to_string(),
                param_type: "string".to_string(),
                description: "New service name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("webserver".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_rename",
            "parameters": {
                "old_name": "nginx",
                "new_name": "webserver"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx renamed to webserver".to_string()
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
        let old_name = parameters
            .get("old_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'old_name' parameter"))?;
        let new_name = parameters
            .get("new_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'new_name' parameter"))?;
        rename_service(old_name, new_name)?;
        Ok(format!("Service {} renamed to {}", old_name, new_name))
    }
}
