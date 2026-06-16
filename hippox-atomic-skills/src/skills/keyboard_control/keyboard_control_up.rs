// keyboard_control/keyboard_control_up.rs
//! Keyboard up skill - release a held key

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{get_key_code, send_key_up};
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct KeyboardControlUpSkill;

#[async_trait::async_trait]
impl Skill for KeyboardControlUpSkill {
    fn name(&self) -> &str {
        "keyboard_control_up"
    }

    fn description(&self) -> &str {
        "Release a keyboard key that was being held down"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to release a key that was previously pressed with 'keyboard_control_down'."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "key".to_string(),
            param_type: "string".to_string(),
            description: "Key to release (e.g., 'shift', 'ctrl', 'alt')".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("shift".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "keyboard_control_up",
            "parameters": {
                "key": "shift"
            }
        })
    }

    fn example_output(&self) -> String {
        "Key up: shift".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Keyboard
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let key = parameters
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'key' parameter"))?;

        let key_code = get_key_code(key).ok_or_else(|| anyhow::anyhow!("Unknown key: {}", key))?;

        send_key_up(key_code)?;

        Ok(format!("Key up: {}", key))
    }
}
