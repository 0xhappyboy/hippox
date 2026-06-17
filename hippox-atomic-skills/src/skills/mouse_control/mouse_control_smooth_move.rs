// mouse_control/mouse_control_smooth_move.rs
//! Mouse smooth move skill

use super::common::smooth_move_to;
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
pub struct MouseControlSmoothMoveSkill;

#[async_trait::async_trait]
impl Skill for MouseControlSmoothMoveSkill {
    fn name(&self) -> &str {
        "mouse_control_smooth_move"
    }

    fn description(&self) -> &str {
        "Move mouse cursor smoothly to target with acceleration"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill for more natural-looking mouse movements. The cursor will accelerate and decelerate smoothly."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "Target X coordinate".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Target Y coordinate".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(300.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "duration_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Movement duration in milliseconds".to_string(),
                required: false,
                default: Some(Value::Number(200.into())),
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "mouse_control_smooth_move",
            "parameters": {
                "x": 500,
                "y": 300,
                "duration_ms": 300
            }
        })
    }

    fn example_output(&self) -> String {
        "Mouse smoothly moved to (500, 300) in 300ms".to_string()
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

        let duration_ms = parameters
            .get("duration_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(200);

        smooth_move_to(x, y, duration_ms).await?;

        Ok(format!(
            "Mouse smoothly moved to ({}, {}) in {}ms",
            x, y, duration_ms
        ))
    }
}
