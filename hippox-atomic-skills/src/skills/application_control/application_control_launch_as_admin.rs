// application_control/application_control_launch_as_admin.rs
//! Application launch as admin skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::launch_as_admin;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct ApplicationControlLaunchAsAdminSkill;

#[async_trait::async_trait]
impl Skill for ApplicationControlLaunchAsAdminSkill {
    fn name(&self) -> &str {
        "application_control_launch_as_admin"
    }

    fn description(&self) -> &str {
        "Launch an application with administrator privileges"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to run an application as administrator. May trigger UAC prompt."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the application executable".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("cmd.exe".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "args".to_string(),
                param_type: "array".to_string(),
                description: "Command-line arguments".to_string(),
                required: false,
                default: Some(Value::Array(vec![])),
                example: Some(json!(["/c", "echo hello"])),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "application_control_launch_as_admin",
            "parameters": {
                "path": "cmd.exe",
                "args": ["/c", "echo hello"]
            }
        })
    }

    fn example_output(&self) -> String {
        "Application launched as admin with PID: 12345".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Application
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let args = parameters
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let pid = launch_as_admin(path, &args)?;
        Ok(format!("Application launched as admin with PID: {}", pid))
    }
}
