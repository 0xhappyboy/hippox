//! Service reset failure count skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::reset_failure_count;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct ServiceResetFailureCountSkill;

#[async_trait::async_trait]
impl Skill for ServiceResetFailureCountSkill {
    fn name(&self) -> &str {
        "service_reset_failure_count"
    }

    fn description(&self) -> &str {
        "Reset service failure count"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to reset the failure counter for a service."
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
            "action": "service_reset_failure_count",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx failure count reset".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemServices
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let service_name = parameters
            .get("service_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'service_name' parameter"))?;
        reset_failure_count(service_name)?;
        Ok(format!("Service {} failure count reset", service_name))
    }
}