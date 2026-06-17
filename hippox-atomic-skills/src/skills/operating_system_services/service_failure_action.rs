//! Service failure action skill

use super::common::set_failure_action;
use crate::{SkillCallback, SkillContext};
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ServiceFailureActionSkill;

#[async_trait::async_trait]
impl Skill for ServiceFailureActionSkill {
    fn name(&self) -> &str {
        "service_failure_action"
    }

    fn description(&self) -> &str {
        "Set action to take when service fails"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to configure what happens when a service fails. Options: restart, stop, ignore"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "service_name".to_string(),
                param_type: "string".to_string(),
                description: "Name of the service".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("nginx".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "action".to_string(),
                param_type: "string".to_string(),
                description: "Action on failure: restart, stop, ignore".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("restart".to_string())),
                enum_values: Some(vec![
                    "restart".to_string(),
                    "stop".to_string(),
                    "ignore".to_string(),
                ]),
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_failure_action",
            "parameters": {
                "service_name": "nginx",
                "action": "restart"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx failure action set to restart".to_string()
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
        let action = parameters
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'action' parameter"))?;
        set_failure_action(service_name, action)?;
        Ok(format!(
            "Service {} failure action set to {}",
            service_name, action
        ))
    }
}
