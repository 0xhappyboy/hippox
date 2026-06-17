// mouse_control/mouse_control_move_to.rs
//! Mouse move to skill

use crate::SkillCallback;
use crate::SkillContext;
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::set_mouse_position;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct MouseControlMoveToSkill;

#[async_trait::async_trait]
impl Skill for MouseControlMoveToSkill {
    fn name(&self) -> &str {
        "mouse_control_move_to"
    }

    fn description(&self) -> &str {
        "Move mouse cursor to specified coordinates"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to move the mouse to an absolute position on the screen."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "X coordinate to move to".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Y coordinate to move to".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(300.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "mouse_control_move_to",
            "parameters": {
                "x": 500,
                "y": 300
            }
        })
    }

    fn example_output(&self) -> String {
        "Mouse moved to (500, 300)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Mouse
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let x = parameters
            .get("x")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'x' parameter"))? as i32;

        let y = parameters
            .get("y")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'y' parameter"))? as i32;

        set_mouse_position(x, y)?;

        Ok(format!("Mouse moved to ({}, {})", x, y))
    }
}
