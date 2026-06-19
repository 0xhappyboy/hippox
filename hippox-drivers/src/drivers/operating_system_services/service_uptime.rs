//! Service uptime Driver

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_service_uptime;
use crate::{DriverCallback, DriverContext};
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct ServiceUptimeDriver;

#[async_trait::async_trait]
impl Driver for ServiceUptimeDriver {
    fn name(&self) -> &str {
        "service_uptime"
    }

    fn description(&self) -> &str {
        "Get the uptime of a service"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see how long a service has been running."
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
            "action": "service_uptime",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx uptime: 2 days, 3 hours, 15 minutes".to_string()
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
        let uptime = get_service_uptime(service_name)?;
        if let Some(uptime) = uptime {
            Ok(format!("Service {} uptime: {}", service_name, uptime))
        } else {
            Ok(format!(
                "Service {} is not running or no uptime information available",
                service_name
            ))
        }
    }
}
