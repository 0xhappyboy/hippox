//! Window minimize skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{find_window, show_window};
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WindowControlMinimizeSkill;

#[async_trait::async_trait]
impl Skill for WindowControlMinimizeSkill {
    fn name(&self) -> &str {
        "window_control_minimize"
    }

    fn description(&self) -> &str {
        "Minimize a specified window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to minimize a window by title or process name"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Window title (partial match)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("微信".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "process".to_string(),
                param_type: "string".to_string(),
                description: "Process name (e.g., WeChat.exe)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("WeChat.exe".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_minimize",
            "parameters": {
                "title": "微信"
            }
        })
    }

    fn example_output(&self) -> String {
        "Window minimized".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Window
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());

        let window_id = find_window(title, process)?;

        #[cfg(target_os = "windows")]
        show_window(window_id, 6)?; // SW_MINIMIZE = 6

        #[cfg(not(target_os = "windows"))]
        {
            let _ = window_id;
            anyhow::bail!("Minimize not implemented on this platform");
        }

        Ok("Window minimized".to_string())
    }
}
