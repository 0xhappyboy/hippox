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

use crate::executors::types::{Skill, SkillParameter};

/// Skill for retrieving text content from the system clipboard
///
/// This skill reads the current content of the system clipboard and returns
/// it as a string. It works across all major operating systems using
/// platform-specific clipboard access methods.
///
/// # Platform Notes
///
/// - **macOS**: Requires `pbpaste` command (built-in)
/// - **Linux**: Requires `xclip` or `xsel` to be installed
/// - **Windows**: Uses `clipboard_win` crate with native Windows API
///
/// # Example
///
/// ```json
/// {
///     "action": "clipboard_get",
///     "parameters": {}
/// }
/// ```
#[derive(Debug)]
pub struct ClipboardGetSkill;

#[async_trait::async_trait]
impl Skill for ClipboardGetSkill {
    /// Returns the unique name of this skill
    fn name(&self) -> &str {
        "clipboard_get"
    }

    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Get text content from system clipboard"
    }

    /// Returns a hint about when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to retrieve text that was copied to clipboard"
    }

    /// Returns the list of parameters accepted by this skill
    ///
    /// This skill takes no parameters.
    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    /// Returns an example JSON call for this skill
    fn example_call(&self) -> Value {
        json!({
            "action": "clipboard_get",
            "parameters": {}
        })
    }

    /// Returns an example output string for this skill
    fn example_output(&self) -> String {
        "Text content from clipboard".to_string()
    }

    /// Returns the category of this skill
    fn category(&self) -> &str {
        "system"
    }

    /// Executes the clipboard get operation
    ///
    /// This method retrieves the current clipboard content using platform-specific
    /// implementations.
    ///
    /// # Arguments
    ///
    /// * `_parameters` - Unused, as this skill takes no parameters
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The clipboard text content on success, or an error
    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("pbpaste")
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to get clipboard content: {}", e))?;
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to get clipboard content: {}", stderr)
            }
        }
        #[cfg(target_os = "linux")]
        {
            let output = std::process::Command::new("xclip")
                .arg("-selection")
                .arg("clipboard")
                .arg("-o")
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    return Ok(String::from_utf8_lossy(&out.stdout).to_string());
                }
            }
            let output = std::process::Command::new("xsel")
                .arg("--clipboard")
                .arg("--output")
                .output()
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to get clipboard content. Install xclip or xsel: {}",
                        e
                    )
                })?;
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to get clipboard content: {}", stderr)
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

/// Skill for setting text content to the system clipboard
///
/// This skill copies the provided text content to the system clipboard,
/// making it available for pasting in other applications. It works across
/// all major operating systems using platform-specific clipboard access methods.
///
/// # Platform Notes
///
/// - **macOS**: Uses `pbcopy` command-line tool (built-in)
/// - **Linux**: Uses `xclip` or `xsel` (fallback) command-line tools
/// - **Windows**: Uses `clipboard_win` crate with native Windows API
///
/// # Parameters
///
/// * `content` (required, string): The text content to copy to the clipboard
///
/// # Example
///
/// ```json
/// {
///     "action": "clipboard_set",
///     "parameters": {
///         "content": "Hello, World!"
///     }
/// }
/// ```
#[derive(Debug)]
pub struct ClipboardSetSkill;

#[async_trait::async_trait]
impl Skill for ClipboardSetSkill {
    /// Returns the unique name of this skill
    fn name(&self) -> &str {
        "clipboard_set"
    }

    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Set text content to system clipboard"
    }

    /// Returns a hint about when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to copy text to clipboard"
    }

    /// Returns the list of parameters accepted by this skill
    ///
    /// This skill requires a single parameter: `content` (string).
    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "content".to_string(),
            param_type: "string".to_string(),
            description: "Text content to copy to clipboard".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Hello, World!".to_string())),
            enum_values: None,
        }]
    }

    /// Returns an example JSON call for this skill
    fn example_call(&self) -> Value {
        json!({
            "action": "clipboard_set",
            "parameters": {
                "content": "Hello, World!"
            }
        })
    }

    /// Returns an example output string for this skill
    fn example_output(&self) -> String {
        "Content copied to clipboard".to_string()
    }

    /// Returns the category of this skill
    fn category(&self) -> &str {
        "system"
    }

    /// Executes the clipboard set operation
    ///
    /// This method copies the provided text to the clipboard using
    /// platform-specific implementations.
    ///
    /// # Arguments
    ///
    /// * `parameters` - Must contain a "content" key with the text to copy
    ///
    /// # Returns
    ///
    /// * `Result<String>` - Success message on success, or an error
    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let content = parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;
        #[cfg(target_os = "macos")]
        {
            let mut child = std::process::Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| anyhow::anyhow!("Failed to spawn pbcopy: {}", e))?;
            let mut stdin = child.stdin.take().expect("Failed to get stdin");
            use std::io::Write;
            stdin.write_all(content.as_bytes())?;
            drop(stdin);
            let output = child.wait_with_output()?;
            if output.status.success() {
                Ok("Content copied to clipboard".to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to copy to clipboard: {}", stderr)
            }
        }
        #[cfg(target_os = "linux")]
        {
            let mut child = std::process::Command::new("xclip")
                .arg("-selection")
                .arg("clipboard")
                .stdin(std::process::Stdio::piped())
                .spawn();
            if let Ok(mut child) = child {
                let mut stdin = child.stdin.take().expect("Failed to get stdin");
                use std::io::Write;
                stdin.write_all(content.as_bytes())?;
                drop(stdin);
                let output = child.wait_with_output()?;
                if output.status.success() {
                    return Ok("Content copied to clipboard".to_string());
                }
            }
            let mut child = std::process::Command::new("xsel")
                .arg("--clipboard")
                .arg("--input")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| {
                    anyhow::anyhow!("Failed to spawn xsel. Install xclip or xsel: {}", e)
                })?;
            let mut stdin = child.stdin.take().expect("Failed to get stdin");
            use std::io::Write;
            stdin.write_all(content.as_bytes())?;
            drop(stdin);
            let output = child.wait_with_output()?;
            if output.status.success() {
                Ok("Content copied to clipboard".to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to copy to clipboard: {}", stderr)
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

    /// Validates the parameters for the clipboard set operation
    ///
    /// # Arguments
    ///
    /// * `parameters` - The parameters to validate
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if parameters are valid, otherwise an error
    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;
        Ok(())
    }
}

/// Skill for clearing the system clipboard content
///
/// This skill clears the system clipboard by setting its content to an empty string.
/// It works by delegating to the `ClipboardSetSkill` with an empty content string.
///
/// # Platform Notes
///
/// Same platform requirements as `ClipboardSetSkill` (depends on the underlying
/// implementation).
///
/// # Example
///
/// ```json
/// {
///     "action": "clipboard_clear",
///     "parameters": {}
/// }
/// ```
#[derive(Debug)]
pub struct ClipboardClearSkill;

#[async_trait::async_trait]
impl Skill for ClipboardClearSkill {
    /// Returns the unique name of this skill
    fn name(&self) -> &str {
        "clipboard_clear"
    }

    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Clear system clipboard content"
    }

    /// Returns a hint about when to use this skill
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to clear clipboard content"
    }

    /// Returns the list of parameters accepted by this skill
    ///
    /// This skill takes no parameters.
    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    /// Returns an example JSON call for this skill
    fn example_call(&self) -> Value {
        json!({
            "action": "clipboard_clear",
            "parameters": {}
        })
    }

    /// Returns an example output string for this skill
    fn example_output(&self) -> String {
        "Clipboard cleared".to_string()
    }

    /// Returns the category of this skill
    fn category(&self) -> &str {
        "system"
    }

    /// Executes the clipboard clear operation
    ///
    /// This method clears the clipboard by setting its content to an empty string.
    ///
    /// # Arguments
    ///
    /// * `_parameters` - Unused, as this skill takes no parameters
    ///
    /// # Returns
    ///
    /// * `Result<String>` - Success message on success, or an error
    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        ClipboardSetSkill
            .execute(&{
                let mut params = HashMap::new();
                params.insert("content".to_string(), Value::String(String::new()));
                params
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test parameter validation for ClipboardSetSkill
    #[test]
    fn test_clipboard_set_validation() {
        let skill = ClipboardSetSkill;
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
        let mut null_params = HashMap::new();
        null_params.insert("content".to_string(), Value::Null);
        assert!(skill.validate(&null_params).is_err());
    }

    /// Test that skills return correct metadata
    #[test]
    fn test_skill_metadata() {
        let get_skill = ClipboardGetSkill;
        let set_skill = ClipboardSetSkill;
        let clear_skill = ClipboardClearSkill;
        assert_eq!(get_skill.name(), "clipboard_get");
        assert_eq!(set_skill.name(), "clipboard_set");
        assert_eq!(clear_skill.name(), "clipboard_clear");
        assert_eq!(get_skill.category(), "system");
        assert_eq!(set_skill.category(), "system");
        assert_eq!(clear_skill.category(), "system");
        assert!(!get_skill.description().is_empty());
        assert!(!set_skill.description().is_empty());
        assert!(!clear_skill.description().is_empty());
        assert!(!get_skill.usage_hint().is_empty());
        assert!(!set_skill.usage_hint().is_empty());
        assert!(!clear_skill.usage_hint().is_empty());
    }

    /// Test parameter definitions for each skill
    #[test]
    fn test_skill_parameters() {
        let get_skill = ClipboardGetSkill;
        let set_skill = ClipboardSetSkill;
        let clear_skill = ClipboardClearSkill;
        assert_eq!(get_skill.parameters().len(), 0);
        let set_params = set_skill.parameters();
        assert_eq!(set_params.len(), 1);
        assert_eq!(set_params[0].name, "content");
        assert_eq!(set_params[0].param_type, "string");
        assert!(set_params[0].required);
        assert_eq!(clear_skill.parameters().len(), 0);
    }

    /// Test example call and output formatting
    #[test]
    fn test_skill_examples() {
        let get_skill = ClipboardGetSkill;
        let set_skill = ClipboardSetSkill;
        let clear_skill = ClipboardClearSkill;
        let get_call = get_skill.example_call();
        assert_eq!(get_call["action"], "clipboard_get");
        let set_call = set_skill.example_call();
        assert_eq!(set_call["action"], "clipboard_set");
        assert!(set_call["parameters"]["content"].is_string());
        let clear_call = clear_skill.example_call();
        assert_eq!(clear_call["action"], "clipboard_clear");
        assert!(!get_skill.example_output().is_empty());
        assert!(!set_skill.example_output().is_empty());
        assert!(!clear_skill.example_output().is_empty());
    }
}
