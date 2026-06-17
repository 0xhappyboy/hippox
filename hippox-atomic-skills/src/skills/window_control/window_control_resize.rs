//! Window resize skill

use super::common::{find_window, get_window_rect, set_window_pos};
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
pub struct WindowControlResizeSkill;

#[async_trait::async_trait]
impl Skill for WindowControlResizeSkill {
    fn name(&self) -> &str {
        "window_control_resize"
    }

    fn description(&self) -> &str {
        "Resize a specified window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to change the size of a window"
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
                name: "width".to_string(),
                param_type: "integer".to_string(),
                description: "New width in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(800.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "height".to_string(),
                param_type: "integer".to_string(),
                description: "New height in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(600.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_resize",
            "parameters": {
                "title": "记事本",
                "width": 800,
                "height": 600
            }
        })
    }

    fn example_output(&self) -> String {
        "Window resized to 800x600".to_string()
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
        let width = parameters
            .get("width")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing width"))? as u32;
        let height = parameters
            .get("height")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing height"))? as u32;

        let window_id = find_window(title, process)?;
        let rect = get_window_rect(window_id)?;

        set_window_pos(window_id, rect.x, rect.y, width, height)?;

        Ok(format!("Window resized to {}x{}", width, height))
    }
}
