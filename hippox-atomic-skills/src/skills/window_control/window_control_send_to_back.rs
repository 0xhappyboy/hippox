//! Window send to back skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::find_window;
use crate::{
    SkillCallback, SkillCategory, SkillContext, types::{Skill, SkillParameter}
};

#[derive(Debug)]
pub struct WindowControlSendToBackSkill;

#[async_trait::async_trait]
impl Skill for WindowControlSendToBackSkill {
    fn name(&self) -> &str {
        "window_control_send_to_back"
    }

    fn description(&self) -> &str {
        "Send a window to the back (behind other windows)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to send a window behind all other windows"
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
                description: "Process name".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("WeChat.exe".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_send_to_back",
            "parameters": {
                "title": "微信"
            }
        })
    }

    fn example_output(&self) -> String {
        "Window sent to back".to_string()
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
        {
            use windows::Win32::UI::WindowsAndMessaging::{
                HWND_BOTTOM, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos,
            };

            unsafe {
                let hwnd = super::common::u64_to_hwnd(window_id);
                let _ = SetWindowPos(hwnd, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = window_id;
            anyhow::bail!("Send to back not implemented on this platform");
        }

        Ok("Window sent to back".to_string())
    }
}
