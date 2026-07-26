//! Service recent Driver - list recently started services
use std::collections::HashMap;
use serde_json::{json, Value};
use tracing::{debug, info};
use super::common::get_recently_started_services;
use crate::{
    types::{Driver, DriverParameter},
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
};
/// Driver for listing recently started services
#[derive(Debug)]
pub struct ServiceRecentDriver;
#[async_trait::async_trait]
impl Driver for ServiceRecentDriver {
    fn name(&self) -> &str {
        return "service_recent";
    }
    fn description(&self) -> &str {
        return "List recently started services";
    }
    fn usage_hint(&self) -> &str {
        return "Use this skill to see recently started services.";
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "limit".to_string(),
            param_type: "integer".to_string(),
            description: "Number of services to show (default: 10)".to_string(),
            required: false,
            default: Some(Value::Number(10.into())),
            example: Some(Value::Number(20.into())),
            enum_values: None,
        }];
    }
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "service_recent",
            "parameters": {
                "limit": 10
            }
        }));
    }
    fn example_output(&self) -> String {
        return "Recently started services:\n1. ssh (started: 2024-01-01 00:00:00)\n2. nginx (started: 2024-01-01 00:00:01)".to_string();
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
        debug!("Executing service_recent driver");
        let limit = parameters
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;
        let services = get_recently_started_services(limit).map_err(|e| {
            debug!("Failed to get recently started services: {}", e);
            return DriverError::execution(format!("Failed to get recently started services: {}", e));
        })?;
        if services.is_empty() {
            info!("No recently started services found");
            return Ok("No recently started services found".to_string());
        }
        let mut result = format!("Recently started services:\n");
        for (i, svc) in services.iter().enumerate() {
            let uptime = svc.uptime.as_deref().unwrap_or("unknown");
            result.push_str(&format!("{}. {} (started: {})\n", i + 1, svc.name, uptime));
        }
        info!("Found {} recently started services", services.len());
        return Ok(result);
    }
}