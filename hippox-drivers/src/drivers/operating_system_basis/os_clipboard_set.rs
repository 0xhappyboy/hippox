//! Clipboard set driver
//!
//! This driver provides functionality to set text content to the system clipboard.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for setting clipboard content
#[derive(Debug)]
pub struct ClipboardSetDriver;
#[async_trait::async_trait]
impl Driver for ClipboardSetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_clipboard_set"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set text content to system clipboard"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to copy text to clipboard"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "content".to_string(),
            param_type: "string".to_string(),
            description: "Text content to copy to clipboard".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Hello, World!".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_clipboard_set",
            "parameters": {
                "content": "Hello, World!"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Content copied to clipboard".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemBasis;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing os_clipboard_set driver");
        let content = parameters.get("content").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("content"))?;
        #[cfg(target_os = "macos")]
        {
            debug!("Setting clipboard content on macOS");
            let result = exec_with_stdin_async("pbcopy", &[], content, None)
                .await
                .map_err(|e| DriverError::execution(format!("Failed to set clipboard content on macOS: {}", e)))?;
            if result.success {
                info!("Clipboard content set successfully on macOS");
                return Ok("Content copied to clipboard".to_string());
            } else {
                return Err(DriverError::execution(format!("Failed to copy to clipboard: {}", result.stderr)));
            }
        }
        #[cfg(target_os = "linux")]
        {
            debug!("Setting clipboard content on Linux");
            let result = exec_with_stdin_async("xclip", &["-selection", "clipboard"], content, None).await;
            if let Ok(r) = result {
                if r.success {
                    info!("Clipboard content set successfully using xclip on Linux");
                    return Ok("Content copied to clipboard".to_string());
                }
            }
            debug!("Trying xsel as fallback on Linux");
            let result = exec_with_stdin_async("xsel", &["--clipboard", "--input"], content, None)
                .await
                .map_err(|e| DriverError::execution(format!("Failed to set clipboard content on Linux: {}", e)))?;
            if result.success {
                info!("Clipboard content set successfully using xsel on Linux");
                return Ok("Content copied to clipboard".to_string());
            } else {
                return Err(DriverError::execution(format!("Failed to copy to clipboard. Install xclip or xsel: {}", result.stderr)));
            }
        }
        #[cfg(target_os = "windows")]
        {
            debug!("Setting clipboard content on Windows");
            use clipboard_win::set_clipboard_string;
            set_clipboard_string(content).map_err(|e| DriverError::execution(format!("Failed to set clipboard content on Windows: {:?}", e)))?;
            info!("Clipboard content set successfully on Windows");
            return Ok("Content copied to clipboard".to_string());
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            debug!("Clipboard operation not supported on this platform");
            return Err(DriverError::execution("Clipboard operation not supported on this platform"));
        }
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating os_clipboard_set parameters");
        parameters.get("content").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("content"))?;
        info!("os_clipboard_set validation passed");
        return Ok(());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_clipboard_set_metadata() {
        let driver = ClipboardSetDriver;
        assert_eq!(driver.name(), "os_clipboard_set");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
