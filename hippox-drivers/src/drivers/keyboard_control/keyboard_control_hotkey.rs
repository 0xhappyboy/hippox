// keyboard_control/keyboard_control_hotkey.rs
//! Keyboard hotkey skill - system-level hotkeys

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
pub struct KeyboardControlHotkeyDriver;

#[async_trait::async_trait]
impl Driver for KeyboardControlHotkeyDriver {
    fn name(&self) -> &str {
        "keyboard_control_hotkey"
    }

    fn description(&self) -> &str {
        "Send system-level hotkeys (e.g., Win+R, Win+E, Alt+Tab)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to send system-wide hotkeys that control the operating system."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "hotkey".to_string(),
            param_type: "string".to_string(),
            description: "Hotkey combination (e.g., 'Win+R', 'Win+E', 'Alt+Tab')".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Win+R".to_string())),
            enum_values: Some(vec![
                "Win+R".to_string(),
                "Win+E".to_string(),
                "Win+D".to_string(),
                "Win+L".to_string(),
                "Win+S".to_string(),
                "Alt+Tab".to_string(),
                "Ctrl+Alt+Delete".to_string(),
                "Alt+F4".to_string(),
            ]),
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "keyboard_control_hotkey",
            "parameters": {
                "hotkey": "Win+R"
            }
        })
    }

    fn example_output(&self) -> String {
        "Hotkey sent: Win+R".to_string()
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
        let hotkey = parameters
            .get("hotkey")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'hotkey' parameter"))?;

        send_shortcut(hotkey)?;

        Ok(format!("Hotkey sent: {}", hotkey))
    }
}
