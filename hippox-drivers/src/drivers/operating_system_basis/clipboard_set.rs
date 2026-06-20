//! Clipboard set driver
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
pub struct ClipboardSetDriver;
#[async_trait::async_trait]
impl Driver for ClipboardSetDriver {
    fn name(&self) -> &str {
        "clipboard_set"
    }
    fn description(&self) -> &str {
        "Set text content to system clipboard"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to copy text to clipboard"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "content".to_string(),
            param_type: "string".to_string(),
            description: "Text content to copy to clipboard".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Hello, World!".to_string())),
            enum_values: None,
        }]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "clipboard_set",
            "parameters": {
                "content": "Hello, World!"
            }
        })
    }
    fn example_output(&self) -> String {
        "Content copied to clipboard".to_string()
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemBasis
    }
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        let content = parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;
        #[cfg(target_os = "macos")]
        {
            let result = exec_with_stdin_async("pbcopy", &[], content, None).await?;
            if result.success {
                Ok("Content copied to clipboard".to_string())
            } else {
                anyhow::bail!("Failed to copy to clipboard: {}", result.stderr)
            }
        }
        #[cfg(target_os = "linux")]
        {
            let result =
                exec_with_stdin_async("xclip", &["-selection", "clipboard"], content, None).await;
            if let Ok(r) = result {
                if r.success {
                    return Ok("Content copied to clipboard".to_string());
                }
            }
            let result =
                exec_with_stdin_async("xsel", &["--clipboard", "--input"], content, None).await?;
            if result.success {
                Ok("Content copied to clipboard".to_string())
            } else {
                anyhow::bail!(
                    "Failed to copy to clipboard. Install xclip or xsel: {}",
                    result.stderr
                )
            }
        }
        #[cfg(target_os = "windows")]
        {
            use clipboard_win::set_clipboard_string;
            set_clipboard_string(content)
                .map_err(|e| anyhow::anyhow!("Failed to set clipboard content: {:?}", e))?;
            Ok("Content copied to clipboard".to_string())
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            anyhow::bail!("Clipboard operation not supported on this platform")
        }
    }
    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_clipboard_set_metadata() {
        let driver = ClipboardSetDriver;
        assert_eq!(driver.name(), "clipboard_set");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
