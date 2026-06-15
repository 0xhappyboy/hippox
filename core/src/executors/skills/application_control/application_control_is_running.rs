// application_control/application_control_is_running.rs
//! Application is running check skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::find_process_by_name;
use crate::executors::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct ApplicationControlIsRunningSkill;

#[async_trait::async_trait]
impl Skill for ApplicationControlIsRunningSkill {
    fn name(&self) -> &str {
        "application_control_is_running"
    }

    fn description(&self) -> &str {
        "Check if an application is currently running"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check if a specific application is active."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "name".to_string(),
            param_type: "string".to_string(),
            description: "Application name or process name".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("notepad.exe".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "application_control_is_running",
            "parameters": {
                "name": "notepad.exe"
            }
        })
    }

    fn example_output(&self) -> String {
        "Application is running: true".to_string()
    }

    fn category(&self) -> &str {
        "application_control"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let name = parameters
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;

        let processes = find_process_by_name(name)?;
        let is_running = !processes.is_empty();

        Ok(format!("Application is running: {}", is_running))
    }
}
