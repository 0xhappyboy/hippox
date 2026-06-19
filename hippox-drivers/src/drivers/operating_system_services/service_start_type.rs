//! Service start type Driver

use super::common::get_service_start_type;
use crate::DriverCallback;
use crate::DriverCategory;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ServiceStartTypeDriver;

#[async_trait::async_trait]
impl Driver for ServiceStartTypeDriver {
    fn name(&self) -> &str {
        "service_start_type"
    }

    fn description(&self) -> &str {
        "Get the start type of a service (automatic/manual/disabled)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check how a service is configured to start."
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
            "action": "service_start_type",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx start type: automatic".to_string()
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
        let start_type = get_service_start_type(service_name)?;
        if let Some(start_type) = start_type {
            Ok(format!(
                "Service {} start type: {}",
                service_name, start_type
            ))
        } else {
            Ok(format!(
                "No start type information available for service {}",
                service_name
            ))
        }
    }
}
