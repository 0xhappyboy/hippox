//! Service search Driver - search services by keyword
use super::common::search_services;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for searching services
#[derive(Debug)]
pub struct ServiceSearchDriver;
#[async_trait::async_trait]
impl Driver for ServiceSearchDriver {
    fn name(&self) -> &str {
        return "service_search";
    }
    fn description(&self) -> &str {
        return "Search for services by keyword";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to find services matching a keyword in name or description.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "keyword".to_string(),
            param_type: "string".to_string(),
            description: "Keyword to search for".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("web".to_string())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_search",
            "parameters": {
                "keyword": "web"
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Services matching 'web':\n1. nginx - Web Server\n2. apache2 - Web Server".to_string();
    }
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemServices;
    }
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing service_search driver");
        let keyword = parameters.get("keyword").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'keyword' parameter");
            return DriverError::missing_parameter("keyword");
        })?;
        let services = search_services(keyword).map_err(|e| {
            debug!("Failed to search services with keyword '{}': {}", keyword, e);
            return DriverError::execution(format!("Failed to search services: {}", e));
        })?;
        if services.is_empty() {
            info!("No services found matching '{}'", keyword);
            return Ok(format!("No services found matching '{}'", keyword));
        }
        let mut result = format!("Services matching '{}':\n", keyword);
        for (i, svc) in services.iter().enumerate() {
            result.push_str(&format!("{}. {} - {}\n", i + 1, svc.name, svc.description));
        }
        info!("Found {} services matching '{}'", services.len(), keyword);
        return Ok(result);
    }
}
