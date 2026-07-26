//! Window get selected text driver
//!
//! This driver provides functionality to get the currently selected text in the active window.
use crate::{
    ClipboardGetDriver, DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info};
/// Driver for getting selected text
#[derive(Debug)]
pub struct WindowControlGetSelectedDriver;
#[async_trait::async_trait]
impl Driver for WindowControlGetSelectedDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_get_selected"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the currently selected text in the active window"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get text that the user has selected"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_get_selected"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Selected text: Hello World".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Window;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing window_control_get_selected driver");
        // Copy selected text to clipboard
        #[cfg(target_os = "windows")]
        {
            debug!("Copying selected text on Windows via Ctrl+C");
            use crate::WindowControlSendShortcutDriver;
            let mut params = HashMap::new();
            params.insert("shortcut".to_string(), Value::String("Ctrl+C".to_string()));
            let shortcut_skill = WindowControlSendShortcutDriver;
            let _ = shortcut_skill.execute(&params, callback, context).await;
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        #[cfg(target_os = "macos")]
        {
            debug!("Copying selected text on macOS via Cmd+C");
            let _ = Command::new("osascript").args(["-e", "tell application \"System Events\" to keystroke \"c\" using {command down}"]).output();
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        #[cfg(target_os = "linux")]
        {
            debug!("Copying selected text on Linux via Ctrl+C");
            let _ = Command::new("xdotool").args(["key", "ctrl+c"]).output();
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        // Get clipboard content
        let get_skill = ClipboardGetDriver;
        let result = get_skill.execute(&HashMap::new(), callback, context).await?;
        if result.is_empty() {
            #[cfg(target_os = "linux")]
            {
                debug!("Trying primary selection on Linux");
                let output = Command::new("xclip").args(["-o", "-selection", "primary"]).output();
                if let Ok(output) = output {
                    if let Ok(selected) = String::from_utf8(output.stdout) {
                        if !selected.is_empty() {
                            info!("Selected text retrieved from primary selection: {} chars", selected.len());
                            return Ok(format!("Selected text: {}", selected.trim()));
                        }
                    }
                }
            }
            info!("No text selected");
            return Ok("No text selected".to_string());
        }
        info!("Selected text retrieved: {} chars", result.len());
        return Ok(format!("Selected text: {}", result));
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
