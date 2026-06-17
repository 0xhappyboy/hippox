// application_control/application_control_get_path.rs
//! Application get path skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_app_path;
use crate::{
    SkillCallback, SkillCategory, SkillContext,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct ApplicationControlGetPathSkill;

#[async_trait::async_trait]
impl Skill for ApplicationControlGetPathSkill {
    fn name(&self) -> &str {
        "application_control_get_path"
    }

    fn description(&self) -> &str {
        "Get the full path of an application executable"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to find where an application is installed."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "name".to_string(),
            param_type: "string".to_string(),
            description: "Application name (e.g., 'notepad', 'chrome')".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("notepad".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "application_control_get_path",
            "parameters": {
                "name": "notepad"
            }
        })
    }

    fn example_output(&self) -> String {
        "Application path: C:\\Windows\\System32\\notepad.exe".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Application
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let name = parameters
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;

        let path = get_app_path(name)?;

        Ok(format!("Application path: {}", path))
    }
}
