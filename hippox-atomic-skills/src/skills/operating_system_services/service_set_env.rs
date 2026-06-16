//! Service set environment variable skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::set_service_env;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct ServiceSetEnvSkill;

#[async_trait::async_trait]
impl Skill for ServiceSetEnvSkill {
    fn name(&self) -> &str {
        "service_set_env"
    }

    fn description(&self) -> &str {
        "Set service environment variable"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to set or update an environment variable for a service."
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
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Environment variable name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("MY_VAR".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "value".to_string(),
                param_type: "string".to_string(),
                description: "Environment variable value".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("my_value".to_string())),
                enum_values: None,
            }
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_set_env",
            "parameters": {
                "service_name": "nginx",
                "key": "MY_VAR",
                "value": "my_value"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx environment variable MY_VAR set to my_value".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemServices
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let service_name = parameters
            .get("service_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'service_name' parameter"))?;
        let key = parameters
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'key' parameter"))?;
        let value = parameters
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'value' parameter"))?;
        set_service_env(service_name, key, value)?;
        Ok(format!("Service {} environment variable {} set to {}", service_name, key, value))
    }
}