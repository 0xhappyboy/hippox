//! Service mask Driver

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::mask_service;
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct ServiceMaskDriver;

#[async_trait::async_trait]
impl Driver for ServiceMaskDriver {
    fn name(&self) -> &str {
        "service_mask"
    }

    fn description(&self) -> &str {
        "Mask a service (prevent it from starting)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to completely prevent a service from starting (stronger than disable)."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service to mask".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_mask",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx masked".to_string()
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
        mask_service(service_name)?;
        Ok(format!("Service {} masked", service_name))
    }
}
