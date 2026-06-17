//! Service config path skill

use crate::{SkillCallback, SkillContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_service_config_path;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct ServiceConfigPathSkill;

#[async_trait::async_trait]
impl Skill for ServiceConfigPathSkill {
    fn name(&self) -> &str {
        "service_config_path"
    }

    fn description(&self) -> &str {
        "Get the configuration file path of a service"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to find where a service's configuration is stored."
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
            "action": "service_config_path",
            "parameters": {
                "service_name": "nginx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx config path: /etc/systemd/system/nginx.service".to_string()
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
        let path = get_service_config_path(service_name)?;
        if let Some(path) = path {
            Ok(format!("Service {} config path: {}", service_name, path))
        } else {
            Ok(format!(
                "No configuration file found for service {}",
                service_name
            ))
        }
    }
}
