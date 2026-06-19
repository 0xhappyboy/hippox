//! Service masked list Driver

use super::common::list_masked_services;
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
pub struct ServiceMaskedListDriver;

#[async_trait::async_trait]
impl Driver for ServiceMaskedListDriver {
    fn name(&self) -> &str {
        "service_masked_list"
    }

    fn description(&self) -> &str {
        "List all masked services"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see which services are currently masked."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_masked_list"
        })
    }

    fn example_output(&self) -> String {
        "Masked services:\n1. service1\n2. service2".to_string()
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
        let services = list_masked_services()?;
        if services.is_empty() {
            return Ok("No masked services found".to_string());
        }
        let mut result = format!("Masked services:\n");
        for (i, svc) in services.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, svc));
        }
        Ok(result)
    }
}
