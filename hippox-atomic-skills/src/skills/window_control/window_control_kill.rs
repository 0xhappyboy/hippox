//! Window kill skill (force close)

use super::common::{find_window, kill_window};
use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct WindowControlKillSkill;

#[async_trait::async_trait]
impl Skill for WindowControlKillSkill {
    fn name(&self) -> &str {
        "window_control_kill"
    }

    fn description(&self) -> &str {
        "Force kill a window's process"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to force close a window that won't respond"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Window title (partial match)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("无响应".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "process".to_string(),
                param_type: "string".to_string(),
                description: "Process name".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("notepad.exe".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_kill",
            "parameters": {
                "title": "无响应"
            }
        })
    }

    fn example_output(&self) -> String {
        "Window process killed".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Window
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());

        let window_id = find_window(title, process)?;
        kill_window(window_id)?;

        Ok("Window process killed".to_string())
    }
}
