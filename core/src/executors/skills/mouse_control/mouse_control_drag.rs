// mouse_control/mouse_control_drag.rs
//! Mouse drag skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{MouseButton, mouse_press, mouse_release, set_mouse_position};
use crate::executors::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct MouseControlDragSkill;

#[async_trait::async_trait]
impl Skill for MouseControlDragSkill {
    fn name(&self) -> &str {
        "mouse_control_drag"
    }

    fn description(&self) -> &str {
        "Drag from one position to another"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to drag and drop. Press at start coordinates, move to end coordinates, then release."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "start_x".to_string(),
                param_type: "integer".to_string(),
                description: "Starting X coordinate".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "start_y".to_string(),
                param_type: "integer".to_string(),
                description: "Starting Y coordinate".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "end_x".to_string(),
                param_type: "integer".to_string(),
                description: "Ending X coordinate".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(300.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "end_y".to_string(),
                param_type: "integer".to_string(),
                description: "Ending Y coordinate".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(300.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "button".to_string(),
                param_type: "string".to_string(),
                description: "Mouse button: 'left', 'right', or 'middle'".to_string(),
                required: false,
                default: Some(Value::String("left".to_string())),
                example: Some(Value::String("left".to_string())),
                enum_values: Some(vec![
                    "left".to_string(),
                    "right".to_string(),
                    "middle".to_string(),
                ]),
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "mouse_control_drag",
            "parameters": {
                "start_x": 100,
                "start_y": 100,
                "end_x": 300,
                "end_y": 300,
                "button": "left"
            }
        })
    }

    fn example_output(&self) -> String {
        "Dragged from (100, 100) to (300, 300)".to_string()
    }

    fn category(&self) -> &str {
        "mouse_control"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let start_x = parameters
            .get("start_x")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'start_x' parameter"))?
            as i32;

        let start_y = parameters
            .get("start_y")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'start_y' parameter"))?
            as i32;

        let end_x = parameters
            .get("end_x")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'end_x' parameter"))?
            as i32;

        let end_y = parameters
            .get("end_y")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'end_y' parameter"))?
            as i32;

        let button_str = parameters
            .get("button")
            .and_then(|v| v.as_str())
            .unwrap_or("left");

        let button = match button_str {
            "left" => MouseButton::Left,
            "right" => MouseButton::Right,
            "middle" => MouseButton::Middle,
            _ => MouseButton::Left,
        };
        // Press at start
        mouse_press(button.clone(), start_x, start_y)?;
        // Move to end with small steps
        let steps = 20;
        for i in 1..=steps {
            let t = i as f64 / steps as f64;
            let x = start_x + ((end_x - start_x) as f64 * t) as i32;
            let y = start_y + ((end_y - start_y) as f64 * t) as i32;
            set_mouse_position(x, y)?;
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        // Release at end
        mouse_release(button, end_x, end_y)?;
        Ok(format!(
            "Dragged from ({}, {}) to ({}, {})",
            start_x, start_y, end_x, end_y
        ))
    }
}
