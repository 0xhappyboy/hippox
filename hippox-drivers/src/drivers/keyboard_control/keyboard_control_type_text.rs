// keyboard_control/keyboard_control_type_text.rs
//! Keyboard type text skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::DriverCallback;
use crate::DriverContext;
use super::common::type_text;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct KeyboardControlTypeTextDriver;

#[async_trait::async_trait]
impl Driver for KeyboardControlTypeTextDriver {
    fn name(&self) -> &str {
        "keyboard_control_type_text"
    }

    fn description(&self) -> &str {
        "Type text as keyboard input into the active window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to simulate typing text. Make sure the target window is focused first."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text to type".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello World".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "delay_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Delay between keystrokes in milliseconds".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(50.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "keyboard_control_type_text",
            "parameters": {
                "text": "Hello World",
                "delay_ms": 10
            }
        })
    }

    fn example_output(&self) -> String {
        "Typed: Hello World".to_string()
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
        let text = parameters
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'text' parameter"))?;

        let delay_ms = parameters
            .get("delay_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        if delay_ms > 0 {
            for c in text.chars() {
                type_text(&c.to_string())?;
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }
        } else {
            type_text(text)?;
        }

        Ok(format!("Typed: {}", text))
    }
}
