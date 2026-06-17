// keyboard_control/keyboard_control_modifier.rs
//! Keyboard modifier skill - control modifier keys separately

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::SkillCallback;
use crate::SkillContext;
use super::common::{get_key_code, send_key_down, send_key_up};
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct KeyboardControlModifierSkill;

#[async_trait::async_trait]
impl Skill for KeyboardControlModifierSkill {
    fn name(&self) -> &str {
        "keyboard_control_modifier"
    }

    fn description(&self) -> &str {
        "Control modifier keys (Shift, Ctrl, Alt, Win) separately for advanced combinations"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to press or release individual modifier keys. This is useful for complex keyboard operations."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "modifier".to_string(),
                param_type: "string".to_string(),
                description: "Modifier key (shift, ctrl, alt, win)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ctrl".to_string())),
                enum_values: Some(vec![
                    "shift".to_string(),
                    "ctrl".to_string(),
                    "alt".to_string(),
                    "win".to_string(),
                ]),
            },
            SkillParameter {
                name: "state".to_string(),
                param_type: "string".to_string(),
                description: "State: 'down' to press, 'up' to release".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("down".to_string())),
                enum_values: Some(vec!["down".to_string(), "up".to_string()]),
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "keyboard_control_modifier",
            "parameters": {
                "modifier": "ctrl",
                "state": "down"
            }
        })
    }

    fn example_output(&self) -> String {
        "Modifier ctrl set to down".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Keyboard
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let modifier = parameters
            .get("modifier")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'modifier' parameter"))?;

        let state = parameters
            .get("state")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'state' parameter"))?;

        let key_code = get_key_code(modifier)
            .ok_or_else(|| anyhow::anyhow!("Unknown modifier: {}", modifier))?;

        match state {
            "down" => send_key_down(key_code)?,
            "up" => send_key_up(key_code)?,
            _ => anyhow::bail!("Invalid state: {}. Must be 'down' or 'up'", state),
        }

        Ok(format!("Modifier {} set to {}", modifier, state))
    }
}
