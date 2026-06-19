use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
/// System clipboard management skills
///
/// This module provides skills for interacting with the system clipboard,
/// including getting, setting, and clearing text content across different
/// platforms (Windows, macOS, Linux).
///
/// # Platform Support
///
/// - **macOS**: Uses `pbpaste` and `pbcopy` command-line tools
/// - **Linux**: Uses `xclip` or `xsel` (fallback) command-line tools
/// - **Windows**: Uses `clipboard_win` crate for native clipboard API
/// - **Other platforms**: Not supported, returns error
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for retrieving text content from the system clipboard
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
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
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

/// Driver for setting text content to the system clipboard
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
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
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

/// Driver for clearing the system clipboard content
#[derive(Debug)]
pub struct ClipboardClearDriver;

#[async_trait::async_trait]
impl Driver for ClipboardClearDriver {
    fn name(&self) -> &str {
        "clipboard_clear"
    }

    fn description(&self) -> &str {
        "Clear system clipboard content"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to clear clipboard content"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "clipboard_clear",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        "Clipboard cleared".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystem
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        ClipboardSetDriver
            .execute(
                &{
                    let mut params = HashMap::new();
                    params.insert("content".to_string(), Value::String(String::new()));
                    params
                },
                callback,
                context,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_set_validation() {
        let skill = ClipboardSetDriver;
        let mut valid_params = HashMap::new();
        valid_params.insert(
            "content".to_string(),
            Value::String("test content".to_string()),
        );
        assert!(skill.validate(&valid_params).is_ok());
        let empty_params = HashMap::new();
        assert!(skill.validate(&empty_params).is_err());
        let mut wrong_type_params = HashMap::new();
        wrong_type_params.insert("content".to_string(), Value::Number(123.into()));
        assert!(skill.validate(&wrong_type_params).is_err());
    }

    #[test]
    fn test_skill_metadata() {
        let get_skill = ClipboardGetDriver;
        let set_skill = ClipboardSetDriver;
        let clear_skill = ClipboardClearDriver;
        assert_eq!(get_skill.name(), "clipboard_get");
        assert_eq!(set_skill.name(), "clipboard_set");
        assert_eq!(clear_skill.name(), "clipboard_clear");
        assert_eq!(get_skill.category(), DriverCategory::OperatingSystem);
        assert_eq!(set_skill.category(), DriverCategory::OperatingSystem);
        assert_eq!(clear_skill.category(), DriverCategory::OperatingSystem);
    }

    #[test]
    fn test_skill_parameters() {
        let get_skill = ClipboardGetDriver;
        let set_skill = ClipboardSetDriver;
        let clear_skill = ClipboardClearDriver;
        assert_eq!(get_skill.parameters().len(), 0);
        let set_params = set_skill.parameters();
        assert_eq!(set_params.len(), 1);
        assert_eq!(set_params[0].name, "content");
        assert_eq!(set_params[0].param_type, "string");
        assert!(set_params[0].required);
        assert_eq!(clear_skill.parameters().len(), 0);
    }
}
