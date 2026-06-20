//! Window get selected text skill

use crate::DriverCallback;
use crate::DriverCategory;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct WindowControlGetSelectedDriver;

#[async_trait::async_trait]
impl Driver for WindowControlGetSelectedDriver {
    fn name(&self) -> &str {
        "window_control_get_selected"
    }

    fn description(&self) -> &str {
        "Get the currently selected text in the active window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get text that the user has selected"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_get_selected"
        })
    }

    fn example_output(&self) -> String {
        "Selected text: Hello World".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Window
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        use crate::drivers::operating_system_basis::clipboard::ClipboardGetDriver;
        // First copy selected text
        #[cfg(target_os = "windows")]
        {
            use crate::WindowControlSendShortcutDriver;
            let mut params = HashMap::new();
            params.insert("shortcut".to_string(), Value::String("Ctrl+C".to_string()));
            let shortcut_skill = WindowControlSendShortcutDriver;
            let _ = shortcut_skill.execute(&params, callback, context).await;
        }
        #[cfg(not(target_os = "windows"))]
        {
            // Implement for other platforms
        }
        // Then get clipboard content
        let get_skill = ClipboardGetDriver;
        let result = get_skill
            .execute(&HashMap::new(), callback, context)
            .await?;
        Ok(format!("Selected text: {}", result))
    }
}
