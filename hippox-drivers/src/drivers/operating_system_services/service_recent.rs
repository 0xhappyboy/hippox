//! Service recent Driver

use super::common::get_recently_started_services;
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
pub struct ServiceRecentDriver;

#[async_trait::async_trait]
impl Driver for ServiceRecentDriver {
    fn name(&self) -> &str {
        "service_recent"
    }

    fn description(&self) -> &str {
        "List recently started services"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see recently started services."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "limit".to_string(),
            param_type: "integer".to_string(),
            description: "Number of services to show (default: 10)".to_string(),
            required: false,
            default: Some(Value::Number(10.into())),
            example: Some(Value::Number(20.into())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_recent",
            "parameters": {
                "limit": 10
            }
        })
    }

    fn example_output(&self) -> String {
        "Recently started services:\n1. ssh (started: 2024-01-01 00:00:00)\n2. nginx (started: 2024-01-01 00:00:01)".to_string()
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
        let limit = parameters
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;
        let services = get_recently_started_services(limit)?;
        if services.is_empty() {
            return Ok("No recently started services found".to_string());
        }
        let mut result = format!("Recently started services:\n");
        for (i, svc) in services.iter().enumerate() {
            let uptime = svc.uptime.as_deref().unwrap_or("unknown");
            result.push_str(&format!("{}. {} (started: {})\n", i + 1, svc.name, uptime));
        }
        Ok(result)
    }
}
