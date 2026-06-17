//! Window maximize skill

use super::common::{find_window, show_window};
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
pub struct WindowControlMaximizeSkill;

#[async_trait::async_trait]
impl Skill for WindowControlMaximizeSkill {
    fn name(&self) -> &str {
        "window_control_maximize"
    }

    fn description(&self) -> &str {
        "Maximize a specified window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to maximize a window by title or process name"
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
            "action": "window_control_maximize",
            "parameters": {
                "title": "微信"
            }
        })
    }

    fn example_output(&self) -> String {
        "Window maximized".to_string()
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
        #[cfg(target_os = "windows")]
        show_window(window_id, 3)?; // SW_MAXIMIZE = 3
        #[cfg(not(target_os = "windows"))]
        {
            let _ = window_id;
            anyhow::bail!("Maximize not implemented on this platform");
        }
        Ok("Window maximized".to_string())
    }
}
