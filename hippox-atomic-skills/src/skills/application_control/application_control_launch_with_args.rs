// application_control/application_control_launch_with_args.rs
//! Application launch with arguments skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::launch_app_with_args;
use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct ApplicationControlLaunchWithArgsSkill;

#[async_trait::async_trait]
impl Skill for ApplicationControlLaunchWithArgsSkill {
    fn name(&self) -> &str {
        "application_control_launch_with_args"
    }

    fn description(&self) -> &str {
        "Launch an application with command-line arguments"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to start an application with specific command-line arguments."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the application executable".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("notepad.exe".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "args".to_string(),
                param_type: "array".to_string(),
                description: "Array of command-line arguments".to_string(),
                required: true,
                default: None,
                example: Some(json!(["file.txt"])),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "application_control_launch_with_args",
            "parameters": {
                "path": "notepad.exe",
                "args": ["file.txt"]
            }
        })
    }

    fn example_output(&self) -> String {
        "Application launched with PID: 12345".to_string()
    }

    fn category(&self) -> &str {
        "application_control"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        let args = parameters
            .get("args")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing 'args' parameter"))?;

        let args_str: Vec<String> = args
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        let pid = launch_app_with_args(path, &args_str)?;

        Ok(format!("Application launched with PID: {}", pid))
    }
}
