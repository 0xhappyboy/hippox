//! Service failure count skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_failure_count;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct ServiceFailureCountSkill;

#[async_trait::async_trait]
impl Skill for ServiceFailureCountSkill {
    fn name(&self) -> &str {
        "service_failure_count"
    }

    fn description(&self) -> &str {
        "Get service failure count"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see how many times a service has failed."
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
            "action": "service_failure_count",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx has failed 0 times".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemServices
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let service_name = parameters
            .get("service_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'service_name' parameter"))?;
        let count = get_failure_count(service_name)?;
        if let Some(count) = count {
            Ok(format!("Service {} has failed {} times", service_name, count))
        } else {
            Ok(format!("No failure count available for service {}", service_name))
        }
    }
}