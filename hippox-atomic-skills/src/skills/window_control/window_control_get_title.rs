//! Window get title skill

use super::common::{WindowInfo, find_window};
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
pub struct WindowControlGetTitleSkill;

#[async_trait::async_trait]
impl Skill for WindowControlGetTitleSkill {
    fn name(&self) -> &str {
        "window_control_get_title"
    }

    fn description(&self) -> &str {
        "Get the title of a specified window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the title text of a window"
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
            "action": "window_control_get_title",
            "parameters": {
                "process": "WeChat.exe"
            }
        })
    }

    fn example_output(&self) -> String {
        "Window title: 微信 - 张三".to_string()
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
        let title_match = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());

        let window_id = find_window(title_match, process)?;

        // Get window info
        use super::common::list_windows;
        let windows = list_windows()?;
        let window = windows
            .iter()
            .find(|w| w.id == window_id)
            .ok_or_else(|| anyhow::anyhow!("Window not found"))?;

        Ok(format!("Window title: {}", window.title))
    }
}
