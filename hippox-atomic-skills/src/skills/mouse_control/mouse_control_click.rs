// mouse_control/mouse_control_click.rs
//! Mouse click skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{MouseButton, mouse_click};
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct MouseControlClickSkill;

#[async_trait::async_trait]
impl Skill for MouseControlClickSkill {
    fn name(&self) -> &str {
        "mouse_control_click"
    }

    fn description(&self) -> &str {
        "Click at the current mouse position or specified coordinates"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to perform a left mouse click. Optionally specify x and y coordinates to move before clicking."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "X coordinate to click at".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Y coordinate to click at".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(300.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "mouse_control_click",
            "parameters": {
                "x": 500,
                "y": 300
            }
        })
    }

    fn example_output(&self) -> String {
        "Mouse clicked at (500, 300)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Mouse
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let x = parameters
            .get("x")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);
        let y = parameters
            .get("y")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);

        let (click_x, click_y) = if let (Some(px), Some(py)) = (x, y) {
            (px, py)
        } else {
            let pos = super::common::get_mouse_position()?;
            (pos.x, pos.y)
        };

        mouse_click(MouseButton::Left, click_x, click_y)?;

        Ok(format!("Mouse clicked at ({}, {})", click_x, click_y))
    }
}
