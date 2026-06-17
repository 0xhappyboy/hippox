//! Service stop skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::stop_service;
use crate::{
    SkillCallback, SkillCategory, SkillContext, types::{Skill, SkillParameter}
};

#[derive(Debug)]
pub struct ServiceStopSkill;

#[async_trait::async_trait]
impl Skill for ServiceStopSkill {
    fn name(&self) -> &str {
        "service_stop"
    }

    fn description(&self) -> &str {
        "Stop a running system service"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to stop a running service."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "service_name".to_string(),
            param_type: "string".to_string(),
            description: "Name of the service to stop".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("nginx".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_stop",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx stopped successfully".to_string()
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
        stop_service(service_name)?;
        Ok(format!("Service {} stopped successfully", service_name))
    }
}
