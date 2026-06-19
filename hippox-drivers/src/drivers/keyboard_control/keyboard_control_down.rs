// keyboard_control/keyboard_control_down.rs
//! Keyboard down skill - press and hold a key

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::DriverCallback;
use crate::DriverContext;
use super::common::{get_key_code, send_key_down};
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct KeyboardControlDownDriver;

#[async_trait::async_trait]
impl Driver for KeyboardControlDownDriver {
    fn name(&self) -> &str {
        "keyboard_control_down"
    }

    fn description(&self) -> &str {
        "Press and hold a keyboard key (without releasing)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to hold down a key. The key will remain pressed until 'keyboard_control_up' is called."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "key".to_string(),
            param_type: "string".to_string(),
            description: "Key to press and hold (e.g., 'shift', 'ctrl', 'alt')".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("shift".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "keyboard_control_down",
            "parameters": {
                "key": "shift"
            }
        })
    }

    fn example_output(&self) -> String {
        "Key down: shift".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Keyboard
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let key = parameters
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'key' parameter"))?;

        let key_code = get_key_code(key).ok_or_else(|| anyhow::anyhow!("Unknown key: {}", key))?;

        send_key_down(key_code)?;

        Ok(format!("Key down: {}", key))
    }
}
