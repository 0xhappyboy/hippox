//! Window set always on top skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::find_window;
use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct WindowControlSetAlwaysOnTopSkill;

#[async_trait::async_trait]
impl Skill for WindowControlSetAlwaysOnTopSkill {
    fn name(&self) -> &str {
        "window_control_set_always_on_top"
    }

    fn description(&self) -> &str {
        "Set a window to stay always on top"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to make a window stay on top of all other windows"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Window title (partial match)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("播放器".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "process".to_string(),
                param_type: "string".to_string(),
                description: "Process name".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("mpv.exe".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "enabled".to_string(),
                param_type: "boolean".to_string(),
                description: "Enable always on top (default: true)".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_set_always_on_top",
            "parameters": {
                "title": "播放器",
                "enabled": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Window set to always on top".to_string()
    }

    fn category(&self) -> &str {
        "window_control"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        let enabled = parameters
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let window_id = find_window(title, process)?;

        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::{
                HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos,
            };

            unsafe {
                let hwnd = super::shared::u64_to_hwnd(window_id);
                let flags = SWP_NOMOVE | SWP_NOSIZE;
                if enabled {
                    let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, flags);
                } else {
                    let _ = SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, flags);
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = window_id;
            anyhow::bail!("Set always on top not implemented on this platform");
        }

        Ok(format!("Window set to always on top: {}", enabled))
    }
}
