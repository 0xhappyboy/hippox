use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct ClipboardGetSkill;

#[async_trait::async_trait]
impl Skill for ClipboardGetSkill {
    fn name(&self) -> &str {
        "clipboard_get"
    }

    fn description(&self) -> &str {
        "Get text content from system clipboard"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to retrieve text that was copied to clipboard"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
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

    fn category(&self) -> &str {
        "system"
    }

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

#[derive(Debug)]
pub struct ClipboardSetSkill;

#[async_trait::async_trait]
impl Skill for ClipboardSetSkill {
    fn name(&self) -> &str {
        "clipboard_set"
    }

    fn description(&self) -> &str {
        "Set text content to system clipboard"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to copy text to clipboard"
    }

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

    fn category(&self) -> &str {
        "system"
    }

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

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ClipboardClearSkill;

#[async_trait::async_trait]
impl Skill for ClipboardClearSkill {
    fn name(&self) -> &str {
        "clipboard_clear"
    }

    fn description(&self) -> &str {
        "Clear system clipboard content"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to clear clipboard content"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
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

    fn category(&self) -> &str {
        "system"
    }

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
