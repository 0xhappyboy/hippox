// mouse_control/mouse_control_position_get.rs
//! Mouse position get skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use super::common::get_mouse_position;

#[derive(Debug)]
pub struct MouseControlPositionGetSkill;

#[async_trait::async_trait]
impl Skill for MouseControlPositionGetSkill {
    fn name(&self) -> &str {
        "mouse_control_position_get"
    }

    fn description(&self) -> &str {
        "Get the current mouse cursor position"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to read the current coordinates of the mouse cursor on screen"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "mouse_control_position_get"
        })
    }

    fn example_output(&self) -> String {
        "Mouse position: x=500, y=300".to_string()
    }

    fn category(&self) -> &str {
        "mouse_control"
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let pos = get_mouse_position()?;
        Ok(format!("Mouse position: x={}, y={}", pos.x, pos.y))
    }
}