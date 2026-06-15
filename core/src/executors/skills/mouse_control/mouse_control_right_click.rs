// mouse_control/mouse_control_right_click.rs
//! Mouse right click skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::types::{Skill, SkillParameter};
use super::common::{MouseButton, mouse_click};

#[derive(Debug)]
pub struct MouseControlRightClickSkill;

#[async_trait::async_trait]
impl Skill for MouseControlRightClickSkill {
    fn name(&self) -> &str {
        "mouse_control_right_click"
    }

    fn description(&self) -> &str {
        "Right-click at the current mouse position or specified coordinates"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to open context menus. Optionally specify x and y coordinates."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "X coordinate to right-click at".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Y coordinate to right-click at".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(300.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "mouse_control_right_click",
            "parameters": {
                "x": 500,
                "y": 300
            }
        })
    }

    fn example_output(&self) -> String {
        "Right-clicked at (500, 300)".to_string()
    }

    fn category(&self) -> &str {
        "mouse_control"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let x = parameters.get("x").and_then(|v| v.as_i64()).map(|v| v as i32);
        let y = parameters.get("y").and_then(|v| v.as_i64()).map(|v| v as i32);
        
        let (click_x, click_y) = if let (Some(px), Some(py)) = (x, y) {
            (px, py)
        } else {
            let pos = super::common::get_mouse_position()?;
            (pos.x, pos.y)
        };
        
        mouse_click(MouseButton::Right, click_x, click_y)?;
        
        Ok(format!("Right-clicked at ({}, {})", click_x, click_y))
    }
}