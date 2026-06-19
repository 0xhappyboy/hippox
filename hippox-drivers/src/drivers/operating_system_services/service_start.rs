//! Service start Driver

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::start_service;
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct ServiceStartDriver;

#[async_trait::async_trait]
impl Driver for ServiceStartDriver {
    fn name(&self) -> &str {
        "service_start"
    }

    fn description(&self) -> &str {
        "Start a system service"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to start a service like ssh, nginx, etc."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service to start".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_start",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx started successfully".to_string()
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
        start_service(service_name)?;
        Ok(format!("Service {} started successfully", service_name))
    }
}
