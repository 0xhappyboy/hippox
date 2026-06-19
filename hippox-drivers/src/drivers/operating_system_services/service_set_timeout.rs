//! Service timeout Driver

use super::common::set_startup_timeout;
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
pub struct ServiceSetTimeoutDriver;

#[async_trait::async_trait]
impl Driver for ServiceSetTimeoutDriver {
    fn name(&self) -> &str {
        "service_set_timeout"
    }

    fn description(&self) -> &str {
        "Set service startup timeout"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to configure how long to wait for a service to start."
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
                name: "timeout_seconds".to_string(),
                param_type: "integer".to_string(),
                description: "Timeout in seconds".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_set_timeout",
            "parameters": {
                "service_name": "nginx",
                "timeout_seconds": 60
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx startup timeout set to 60 seconds".to_string()
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
        let timeout = parameters
            .get("timeout_seconds")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'timeout_seconds' parameter"))?
            as u32;
        set_startup_timeout(service_name, timeout)?;
        Ok(format!(
            "Service {} startup timeout set to {} seconds",
            service_name, timeout
        ))
    }
}
