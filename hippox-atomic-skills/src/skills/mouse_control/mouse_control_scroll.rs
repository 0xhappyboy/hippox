// mouse_control/mouse_control_scroll.rs
//! Mouse scroll skill

use super::common::mouse_scroll;
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
pub struct MouseControlScrollSkill;

#[async_trait::async_trait]
impl Skill for MouseControlScrollSkill {
    fn name(&self) -> &str {
        "mouse_control_scroll"
    }

    fn description(&self) -> &str {
        "Scroll the mouse wheel"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to scroll up or down. Positive delta scrolls up, negative scrolls down."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "delta".to_string(),
            param_type: "integer".to_string(),
            description:
                "Scroll amount (positive=up, negative=down). 120 is typical for one click."
                    .to_string(),
            required: true,
            default: None,
            example: Some(Value::Number(120.into())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "mouse_control_scroll",
            "parameters": {
                "delta": 120
            }
        })
    }

    fn example_output(&self) -> String {
        "Scrolled by 120".to_string()
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
        let delta = parameters
            .get("delta")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'delta' parameter"))?
            as i32;

        mouse_scroll(delta)?;

        Ok(format!("Scrolled by {}", delta))
    }
}
