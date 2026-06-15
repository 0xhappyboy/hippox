//! Window find skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::types::{Skill, SkillParameter};
use super::shared::find_window;

#[derive(Debug)]
pub struct WindowControlFindSkill;

#[async_trait::async_trait]
impl Skill for WindowControlFindSkill {
    fn name(&self) -> &str {
        "window_control_find"
    }

    fn description(&self) -> &str {
        "Find a window by title or process name"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get window information by searching by title or process"
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
            "action": "window_control_find",
            "parameters": {
                "title": "微信"
            }
        })
    }

    fn example_output(&self) -> String {
        "Found window: 微信 (ID: 12345678)".to_string()
    }

    fn category(&self) -> &str {
        "window_control"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        
        let window_id = find_window(title, process)?;
        
        use super::shared::list_windows;
        let windows = list_windows()?;
        let window = windows.iter().find(|w| w.id == window_id).ok_or_else(|| anyhow::anyhow!("Window not found"))?;
        
        Ok(format!(
            "Found window: {} (ID: {}, Process: {})",
            window.title, window.id, window.process_name
        ))
    }
}