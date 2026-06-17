//! Window send shortcut skill

use super::common::{find_window, set_foreground_window};
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
pub struct WindowControlSendShortcutSkill;

#[async_trait::async_trait]
impl Skill for WindowControlSendShortcutSkill {
    fn name(&self) -> &str {
        "window_control_send_shortcut"
    }

    fn description(&self) -> &str {
        "Send a keyboard shortcut to a specified window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to send shortcuts like Ctrl+C, Ctrl+V, Alt+Tab"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Window title (partial match)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("记事本".to_string())),
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
            SkillParameter {
                name: "shortcut".to_string(),
                param_type: "string".to_string(),
                description: "Shortcut name (e.g., Ctrl+C, Ctrl+V, Ctrl+S, Alt+F4)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Ctrl+S".to_string())),
                enum_values: Some(vec![
                    "Ctrl+C".to_string(),
                    "Ctrl+V".to_string(),
                    "Ctrl+X".to_string(),
                    "Ctrl+Z".to_string(),
                    "Ctrl+Y".to_string(),
                    "Ctrl+S".to_string(),
                    "Ctrl+A".to_string(),
                    "Alt+F4".to_string(),
                    "Alt+Tab".to_string(),
                    "Enter".to_string(),
                    "Tab".to_string(),
                    "Escape".to_string(),
                    "Delete".to_string(),
                ]),
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_send_shortcut",
            "parameters": {
                "title": "记事本",
                "shortcut": "Ctrl+S"
            }
        })
    }

    fn example_output(&self) -> String {
        "Shortcut sent to window".to_string()
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
        let shortcut = parameters
            .get("shortcut")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing shortcut"))?;

        if let Some(_window_id) = find_window(title, process).ok() {
            // Activate window first
            // set_foreground_window(window_id)?;
        }

        // Use enigo or similar to send shortcut
        let _ = shortcut;

        // Platform-specific shortcut implementation
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::Input::*;
            // Map shortcut string to virtual keys
            match shortcut {
                "Ctrl+C" => {
                    // Send Ctrl+C
                }
                _ => {}
            }
        }

        Ok("Shortcut sent to window (implementation pending)".to_string())
    }
}
