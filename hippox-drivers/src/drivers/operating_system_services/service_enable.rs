//! Service enable Driver

use super::common::enable_service;
use crate::{DriverCallback, DriverContext};
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ServiceEnableDriver;

#[async_trait::async_trait]
impl Driver for ServiceEnableDriver {
    fn name(&self) -> &str {
        "service_enable"
    }

    fn description(&self) -> &str {
        "Enable a service to start automatically on boot"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to set a service to start automatically at system boot."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service to enable".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_enable",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx enabled for auto-start".to_string()
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
        enable_service(service_name)?;
        Ok(format!("Service {} enabled for auto-start", service_name))
    }
}
