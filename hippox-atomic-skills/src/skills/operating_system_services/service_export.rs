//! Service export skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::export_service_config;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct ServiceExportSkill;

#[async_trait::async_trait]
impl Skill for ServiceExportSkill {
    fn name(&self) -> &str {
        "service_export"
    }

    fn description(&self) -> &str {
        "Export service configuration to file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to backup a service's configuration."
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
                name: "output_path".to_string(),
                param_type: "string".to_string(),
                description: "Path to export configuration to".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/nginx.service.backup".to_string())),
                enum_values: None,
            }
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "service_export",
            "parameters": {
                "service_name": "nginx",
                "output_path": "/tmp/nginx.service.backup"
            }
        })
    }

    fn example_output(&self) -> String {
        "Service nginx configuration exported to /tmp/nginx.service.backup".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemServices
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let service_name = parameters
            .get("service_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'service_name' parameter"))?;
        let output_path = parameters
            .get("output_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'output_path' parameter"))?;
        export_service_config(service_name, output_path)?;
        Ok(format!("Service {} configuration exported to {}", service_name, output_path))
    }
}