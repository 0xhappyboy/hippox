//! Service history skill

use super::common::get_service_history;
use crate::SkillCategory;
use crate::{
    SkillCallback, SkillContext,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ServiceHistorySkill;

#[async_trait::async_trait]
impl Skill for ServiceHistorySkill {
    fn name(&self) -> &str {
        "service_history"
    }

    fn description(&self) -> &str {
        "View service change history"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see the change history of a service."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
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
            "action": "service_history",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx history:\n1. 2024-01-01 00:00:00 Service created\n2. 2024-01-02 00:00:00 Configuration updated".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemServices
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let service_name = parameters
            .get("service_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'service_name' parameter"))?;
        let history = get_service_history(service_name)?;
        if history.is_empty() {
            return Ok(format!("No history found for service {}", service_name));
        }
        let mut result = format!("Service {} history:\n", service_name);
        for (i, entry) in history.iter().enumerate() {
            result.push_str(&format!("{}. {}\n", i + 1, entry));
        }
        Ok(result)
    }
}
