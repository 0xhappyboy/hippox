//! Clipboard get driver
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
#[derive(Debug)]
pub struct ClipboardGetDriver;
#[async_trait::async_trait]
impl Driver for ClipboardGetDriver {
    fn name(&self) -> &str {
        "clipboard_get"
    }
    fn description(&self) -> &str {
        "Get text content from system clipboard"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to retrieve text that was copied to clipboard"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "clipboard_get",
            "parameters": {}
        })
    }
    fn example_output(&self) -> String {
        "Text content from clipboard".to_string()
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemBasis
    }
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        #[cfg(target_os = "macos")]
        {
            let result = exec_async("pbpaste", &[], None).await?;
            if result.success {
                Ok(result.stdout)
            } else {
                anyhow::bail!("Failed to get clipboard content: {}", result.stderr)
            }
        }
        #[cfg(target_os = "linux")]
        {
            let result = exec_async("xclip", &["-selection", "clipboard", "-o"], None).await;
            if let Ok(r) = result {
                if r.success {
                    return Ok(r.stdout);
                }
            }
            let result = exec_async("xsel", &["--clipboard", "--output"], None).await?;
            if result.success {
                Ok(result.stdout)
            } else {
                anyhow::bail!(
                    "Failed to get clipboard content. Install xclip or xsel: {}",
                    result.stderr
                )
            }
        }
        #[cfg(target_os = "windows")]
        {
            use clipboard_win::get_clipboard_string;
            let text = get_clipboard_string()
                .map_err(|e| anyhow::anyhow!("Failed to get clipboard content: {:?}", e))?;
            Ok(text)
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            anyhow::bail!("Clipboard operation not supported on this platform")
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_clipboard_get_metadata() {
        let driver = ClipboardGetDriver;
        assert_eq!(driver.name(), "clipboard_get");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}