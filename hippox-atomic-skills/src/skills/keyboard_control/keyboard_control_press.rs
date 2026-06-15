// keyboard_control/keyboard_control_press.rs
//! Keyboard press skill - press and release a key

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};
use super::common::{get_key_code, send_key_press};

#[derive(Debug)]
pub struct KeyboardControlPressSkill;

#[async_trait::async_trait]
impl Skill for KeyboardControlPressSkill {
    fn name(&self) -> &str {
        "keyboard_control_press"
    }

    fn description(&self) -> &str {
        "Press and release a keyboard key"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to press a single key (e.g., Enter, Space, A, 1). The key will be pressed and immediately released."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Key to press (e.g., 'a', 'enter', 'space', 'f1', 'ctrl')".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("enter".to_string())),
                enum_values: Some(vec![
                    "a".to_string(), "b".to_string(), "c".to_string(),
                    "enter".to_string(), "space".to_string(), "tab".to_string(),
                    "esc".to_string(), "f1".to_string(), "f2".to_string(),
                ]),
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "keyboard_control_press",
            "parameters": {
                "key": "enter"
            }
        })
    }

    fn example_output(&self) -> String {
        "Key pressed: enter".to_string()
    }

    fn category(&self) -> &str {
        "keyboard_control"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let key = parameters.get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'key' parameter"))?;
        
        let key_code = get_key_code(key)
            .ok_or_else(|| anyhow::anyhow!("Unknown key: {}", key))?;
        
        send_key_press(key_code)?;
        
        Ok(format!("Key pressed: {}", key))
    }
}