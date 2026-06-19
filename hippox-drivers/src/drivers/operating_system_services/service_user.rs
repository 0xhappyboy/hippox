//! Service user Driver

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_service_user;
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct ServiceUserDriver;

#[async_trait::async_trait]
impl Driver for ServiceUserDriver {
    fn name(&self) -> &str {
        "service_user"
    }

    fn description(&self) -> &str {
        "Get the user/group under which a service runs"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see which user account a service is running as."
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
            "action": "service_user",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx runs as user: www-data, group: www-data".to_string()
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
        let user = get_service_user(service_name)?;
        if let Some(user) = user {
            Ok(format!("Service {} runs as user: {}", service_name, user))
        } else {
            Ok(format!(
                "No user information available for service {}",
                service_name
            ))
        }
    }
}
