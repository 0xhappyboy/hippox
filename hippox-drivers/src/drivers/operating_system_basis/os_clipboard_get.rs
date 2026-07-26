//! Clipboard get driver
//!
//! This driver provides functionality to get text content from the system clipboard.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting clipboard content
#[derive(Debug)]
pub struct ClipboardGetDriver;
#[async_trait::async_trait]
impl Driver for ClipboardGetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_clipboard_get"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get text content from system clipboard"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to retrieve text that was copied to clipboard"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_clipboard_get",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Text content from clipboard".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemBasis;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing os_clipboard_get driver");
        #[cfg(target_os = "macos")]
        {
            debug!("Getting clipboard content on macOS");
            let result = exec_async("pbpaste", &[], None)
                .await
                .map_err(|e| DriverError::execution(format!("Failed to get clipboard content on macOS: {}", e)))?;
            if result.success {
                info!("Clipboard content retrieved successfully on macOS");
                return Ok(result.stdout);
            } else {
                return Err(DriverError::execution(format!("Failed to get clipboard content: {}", result.stderr)));
            }
        }
        #[cfg(target_os = "linux")]
        {
            debug!("Getting clipboard content on Linux");
            let result = exec_async("xclip", &["-selection", "clipboard", "-o"], None).await;
            if let Ok(r) = result {
                if r.success {
                    info!("Clipboard content retrieved successfully using xclip on Linux");
                    return Ok(r.stdout);
                }
            }
            debug!("Trying xsel as fallback on Linux");
            let result = exec_async("xsel", &["--clipboard", "--output"], None)
                .await
                .map_err(|e| DriverError::execution(format!("Failed to get clipboard content on Linux: {}", e)))?;
            if result.success {
                info!("Clipboard content retrieved successfully using xsel on Linux");
                return Ok(result.stdout);
            } else {
                return Err(DriverError::execution(format!("Failed to get clipboard content. Install xclip or xsel: {}", result.stderr)));
            }
        }
        #[cfg(target_os = "windows")]
        {
            debug!("Getting clipboard content on Windows");
            use clipboard_win::get_clipboard_string;
            let text = get_clipboard_string().map_err(|e| DriverError::execution(format!("Failed to get clipboard content on Windows: {:?}", e)))?;
            info!("Clipboard content retrieved successfully on Windows");
            return Ok(text);
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            debug!("Clipboard operation not supported on this platform");
            return Err(DriverError::execution("Clipboard operation not supported on this platform"));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_clipboard_get_metadata() {
        let driver = ClipboardGetDriver;
        assert_eq!(driver.name(), "os_clipboard_get");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
