// keyboard_control/keyboard_control_type_text.rs
//! Keyboard type text skill
//!
//! This driver provides functionality to type text as keyboard input
//! into the active window.
use super::common::type_text;
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};
/// Driver for typing text
#[derive(Debug)]
pub struct KeyboardControlTypeTextDriver;
#[async_trait::async_trait]
impl Driver for KeyboardControlTypeTextDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "keyboard_control_type_text"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Type text as keyboard input into the active window"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to simulate typing text. Make sure the target window is focused first."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
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
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "keyboard_control_type_text",
            "parameters": {
                "text": "Hello World",
                "delay_ms": 10
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Typed: Hello World".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Keyboard;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing keyboard_control_type_text driver");
        let text = parameters.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'text' parameter");
            return crate::DriverError::missing_parameter("text");
        })?;
        let delay_ms = parameters.get("delay_ms").and_then(|v| v.as_u64()).unwrap_or(0);
        debug!("Typing text: '{}' (delay: {}ms)", text, delay_ms);
        if delay_ms > 0 {
            let delay = Duration::from_millis(delay_ms);
            for c in text.chars() {
                type_text(&c.to_string()).map_err(|e| {
                    debug!("Failed to type character: {}", e);
                    return crate::DriverError::execution(format!("Failed to type character: {}", e));
                })?;
                tokio::time::sleep(delay).await;
            }
        } else {
            type_text(text).map_err(|e| {
                debug!("Failed to type text: {}", e);
                return crate::DriverError::execution(format!("Failed to type text: {}", e));
            })?;
        }
        info!("Typed: {}", text);
        return Ok(format!("Typed: {}", text));
    }
}
