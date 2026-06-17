// mouse_control/mouse_control_move_relative.rs
//! Mouse move relative skill

use super::common::{get_mouse_position, set_mouse_position};
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
pub struct MouseControlMoveRelativeSkill;

#[async_trait::async_trait]
impl Skill for MouseControlMoveRelativeSkill {
    fn name(&self) -> &str {
        "mouse_control_move_relative"
    }

    fn description(&self) -> &str {
        "Move mouse cursor relative to current position"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to move the mouse by a delta from its current position."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "dx".to_string(),
                param_type: "integer".to_string(),
                description: "Delta X to move (positive=right, negative=left)".to_string(),
                required: true,
                default: None,
                example: Some(json!(100)),
                enum_values: None,
            },
            SkillParameter {
                name: "dy".to_string(),
                param_type: "integer".to_string(),
                description: "Delta Y to move (positive=down, negative=up)".to_string(),
                required: true,
                default: None,
                example: Some(json!(-50)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "mouse_control_move_relative",
            "parameters": {
                "dx": 100,
                "dy": -50
            }
        })
    }

    fn example_output(&self) -> String {
        "Mouse moved relative by (100, -50)".to_string()
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
        let dx = parameters
            .get("dx")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'dx' parameter"))? as i32;
        let dy = parameters
            .get("dy")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'dy' parameter"))? as i32;
        let current = get_mouse_position()?;
        let new_x = current.x + dx;
        let new_y = current.y + dy;
        set_mouse_position(new_x, new_y)?;
        Ok(format!("Mouse moved relative by ({}, {})", dx, dy))
    }
}
