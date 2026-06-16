//! Service recent skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_recently_started_services;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct ServiceRecentSkill;

#[async_trait::async_trait]
impl Skill for ServiceRecentSkill {
    fn name(&self) -> &str {
        "service_recent"
    }

    fn description(&self) -> &str {
        "List recently started services"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see recently started services."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
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

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemServices
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
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