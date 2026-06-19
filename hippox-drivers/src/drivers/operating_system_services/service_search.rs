//! Service search Driver

use super::common::search_services;
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
pub struct ServiceSearchDriver;

#[async_trait::async_trait]
impl Driver for ServiceSearchDriver {
    fn name(&self) -> &str {
        "service_search"
    }

    fn description(&self) -> &str {
        "Search for services by keyword"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to find services matching a keyword in name or description."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "keyword".to_string(),
            param_type: "string".to_string(),
            description: "Keyword to search for".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("web".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_search",
            "parameters": {
                "keyword": "web"
            }
        })
    }

    fn example_output(&self) -> String {
        "Services matching 'web':\n1. nginx - Web Server\n2. apache2 - Web Server".to_string()
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
        let keyword = parameters
            .get("keyword")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'keyword' parameter"))?;
        let services = search_services(keyword)?;
        if services.is_empty() {
            return Ok(format!("No services found matching '{}'", keyword));
        }
        let mut result = format!("Services matching '{}':\n", keyword);
        for (i, svc) in services.iter().enumerate() {
            result.push_str(&format!("{}. {} - {}\n", i + 1, svc.name, svc.description));
        }
        Ok(result)
    }
}
