// keyboard_control/keyboard_control_shortcut.rs
//! Keyboard shortcut skill - send combination keys

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::DriverCallback;
use crate::DriverContext;
use super::common::send_shortcut;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct KeyboardControlShortcutDriver;

#[async_trait::async_trait]
impl Driver for KeyboardControlShortcutDriver {
    fn name(&self) -> &str {
        "keyboard_control_shortcut"
    }

    fn description(&self) -> &str {
        "Send keyboard shortcut (combination of keys like Ctrl+C, Ctrl+Shift+S)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to send common keyboard shortcuts. Examples: Ctrl+C, Ctrl+V, Ctrl+Shift+S, Alt+F4, Ctrl+Alt+Delete"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "shortcut".to_string(),
            param_type: "string".to_string(),
            description: "Shortcut combination (e.g., 'Ctrl+C', 'Ctrl+Shift+S', 'Alt+F4')"
                .to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Ctrl+S".to_string())),
            enum_values: Some(vec![
                "Ctrl+C".to_string(),
                "Ctrl+V".to_string(),
                "Ctrl+X".to_string(),
                "Ctrl+Z".to_string(),
                "Ctrl+Y".to_string(),
                "Ctrl+S".to_string(),
                "Ctrl+A".to_string(),
                "Alt+F4".to_string(),
                "Ctrl+Alt+Delete".to_string(),
                "Ctrl+Shift+Esc".to_string(),
                "Win+R".to_string(),
            ]),
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "keyboard_control_shortcut",
            "parameters": {
                "shortcut": "Ctrl+S"
            }
        })
    }

    fn example_output(&self) -> String {
        "Shortcut sent: Ctrl+S".to_string()
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
        let shortcut = parameters
            .get("shortcut")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'shortcut' parameter"))?;

        send_shortcut(shortcut)?;

        Ok(format!("Shortcut sent: {}", shortcut))
    }
}
