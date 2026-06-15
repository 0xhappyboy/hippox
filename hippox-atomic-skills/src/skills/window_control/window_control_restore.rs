//! Window restore skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use super::shared::{find_window, show_window};

#[derive(Debug)]
pub struct WindowControlRestoreSkill;

#[async_trait::async_trait]
impl Skill for WindowControlRestoreSkill {
    fn name(&self) -> &str {
        "window_control_restore"
    }

    fn description(&self) -> &str {
        "Restore a minimized or maximized window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to restore a window from minimized or maximized state"
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
            "action": "window_control_restore",
            "parameters": {
                "title": "微信"
            }
        })
    }

    fn example_output(&self) -> String {
        "Window restored".to_string()
    }

    fn category(&self) -> &str {
        "window_control"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        let window_id = find_window(title, process)?;
        #[cfg(target_os = "windows")]
        show_window(window_id, 9)?; // SW_RESTORE = 9
        #[cfg(not(target_os = "windows"))]
        {
            let _ = window_id;
            anyhow::bail!("Restore not implemented on this platform");
        }
        Ok("Window restored".to_string())
    }
}