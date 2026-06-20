//! Window get selected text skill
use crate::ClipboardGetDriver;
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
        #[cfg(target_os = "windows")]
        {
            use crate::WindowControlSendShortcutDriver;
            let mut params = HashMap::new();
            params.insert("shortcut".to_string(), Value::String("Ctrl+C".to_string()));
            let shortcut_skill = WindowControlSendShortcutDriver;
            let _ = shortcut_skill.execute(&params, callback, context).await;
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("osascript")
                .args([
                    "-e",
                    "tell application \"System Events\" to keystroke \"c\" using {command down}",
                ])
                .output();
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("xclip")
                .args(["-o", "-selection", "clipboard"])
                .output();
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let get_skill = ClipboardGetDriver;
        let result = get_skill
            .execute(&HashMap::new(), callback, context)
            .await?;
        if result.is_empty() {
            #[cfg(target_os = "linux")]
            {
                let output = Command::new("xclip")
                    .args(["-o", "-selection", "primary"])
                    .output();
                if let Ok(output) = output {
                    if let Ok(selected) = String::from_utf8(output.stdout) {
                        if !selected.is_empty() {
                            return Ok(format!("Selected text: {}", selected.trim()));
                        }
                    }
                }
            }
            return Ok("No text selected".to_string());
        }
        Ok(format!("Selected text: {}", result))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_window_control_get_selected_metadata() {
        let driver = WindowControlGetSelectedDriver;
        assert_eq!(driver.name(), "window_control_get_selected");
        assert_eq!(driver.category(), DriverCategory::Window);
    }
}
