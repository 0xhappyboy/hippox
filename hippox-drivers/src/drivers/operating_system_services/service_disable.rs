//! Service disable Driver

use super::common::disable_service;
use crate::{DriverCallback, DriverContext};
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ServiceDisableDriver;

#[async_trait::async_trait]
impl Driver for ServiceDisableDriver {
    fn name(&self) -> &str {
        "service_disable"
    }

    fn description(&self) -> &str {
        "Disable a service from starting automatically on boot"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to prevent a service from starting automatically at system boot."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service to disable".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_disable",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx disabled for auto-start".to_string()
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
        disable_service(service_name)?;
        Ok(format!("Service {} disabled for auto-start", service_name))
    }
}
