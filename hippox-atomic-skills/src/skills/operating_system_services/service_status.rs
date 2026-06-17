//! Service status skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_service_status;
use crate::{
    SkillCallback, SkillCategory, SkillContext,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct ServiceStatusSkill;

#[async_trait::async_trait]
impl Skill for ServiceStatusSkill {
    fn name(&self) -> &str {
        "service_status"
    }

    fn description(&self) -> &str {
        "Query the status of a system service"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check if a service is running."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service to check".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_status",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx status: running".to_string()
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
        let status = get_service_status(service_name)?;
        Ok(format!("Service {} status: {}", service_name, status))
    }
}
