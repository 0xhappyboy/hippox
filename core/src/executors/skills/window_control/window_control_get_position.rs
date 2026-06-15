//! Window get position skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::types::{Skill, SkillParameter};
use super::shared::{find_window, get_window_rect};

#[derive(Debug)]
pub struct WindowControlGetPositionSkill;

#[async_trait::async_trait]
impl Skill for WindowControlGetPositionSkill {
    fn name(&self) -> &str {
        "window_control_get_position"
    }

    fn description(&self) -> &str {
        "Get the position and size of a specified window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get window coordinates for mouse operations"
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
            "action": "window_control_get_position",
            "parameters": {
                "title": "微信"
            }
        })
    }

    fn example_output(&self) -> String {
        "Window position: x=100, y=200, width=800, height=600".to_string()
    }

    fn category(&self) -> &str {
        "window_control"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        
        let window_id = find_window(title, process)?;
        let rect = get_window_rect(window_id)?;
        
        Ok(format!("Window position: x={}, y={}, width={}, height={}", rect.x, rect.y, rect.width, rect.height))
    }
}