// mouse_control/mouse_control_release.rs
//! Mouse release skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::types::{Skill, SkillParameter};
use super::common::{MouseButton, mouse_release};

#[derive(Debug)]
pub struct MouseControlReleaseSkill;

#[async_trait::async_trait]
impl Skill for MouseControlReleaseSkill {
    fn name(&self) -> &str {
        "mouse_control_release"
    }

    fn description(&self) -> &str {
        "Release a held mouse button"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to release a mouse button that was held with 'mouse_control_press'."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "button".to_string(),
                param_type: "string".to_string(),
                description: "Mouse button: 'left', 'right', or 'middle'".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("left".to_string())),
                enum_values: Some(vec![
                    "left".to_string(), "right".to_string(), "middle".to_string(),
                ]),
            },
            SkillParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "X coordinate to release at".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Y coordinate to release at".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(300.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "mouse_control_release",
            "parameters": {
                "button": "left"
            }
        })
    }

    fn example_output(&self) -> String {
        "Mouse button left released".to_string()
    }

    fn category(&self) -> &str {
        "mouse_control"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let button_str = parameters.get("button")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'button' parameter"))?;
        
        let button = match button_str {
            "left" => MouseButton::Left,
            "right" => MouseButton::Right,
            "middle" => MouseButton::Middle,
            _ => anyhow::bail!("Unknown button: {}", button_str),
        };
        
        let x = parameters.get("x").and_then(|v| v.as_i64()).map(|v| v as i32);
        let y = parameters.get("y").and_then(|v| v.as_i64()).map(|v| v as i32);
        
        let (release_x, release_y) = if let (Some(px), Some(py)) = (x, y) {
            (px, py)
        } else {
            let pos = super::common::get_mouse_position()?;
            (pos.x, pos.y)
        };
        
        mouse_release(button, release_x, release_y)?;
        
        Ok(format!("Mouse button {} released", button_str))
    }
}