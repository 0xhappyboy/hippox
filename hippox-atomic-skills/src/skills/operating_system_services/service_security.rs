//! Service security skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_service_security;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct ServiceSecuritySkill;

#[async_trait::async_trait]
impl Skill for ServiceSecuritySkill {
    fn name(&self) -> &str {
        "service_security"
    }

    fn description(&self) -> &str {
        "View service security settings"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to see the security context of a service."
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
            "action": "service_security",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx security settings:\nUser: www-data\nGroup: www-data\nProtectSystem: full".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemServices
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let service_name = parameters
            .get("service_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'service_name' parameter"))?;
        let security = get_service_security(service_name)?;
        if security.is_empty() {
            return Ok(format!("No security settings found for service {}", service_name));
        }
        let mut result = format!("Service {} security settings:\n", service_name);
        for (key, value) in security {
            result.push_str(&format!("{}: {}\n", key, value));
        }
        Ok(result)
    }
}