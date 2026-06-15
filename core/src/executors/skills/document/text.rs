use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::{
    ensure_dir, file_exists, read_file_content,
    types::{Skill, SkillParameter},
    validate_path, write_file_content,
};

#[derive(Debug)]
pub struct TextReadSkill;

#[async_trait::async_trait]
impl Skill for TextReadSkill {
    fn name(&self) -> &str {
        "text_read"
    }

    fn description(&self) -> &str {
        "Read plain text file content"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read a plain text file, view logs, or extract content from .txt files"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the text file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("notes.txt".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "encoding".to_string(),
                param_type: "string".to_string(),
                description: "File encoding (utf-8, utf-16, latin1)".to_string(),
                required: false,
                default: Some(Value::String("utf-8".to_string())),
                example: Some(Value::String("utf-8".to_string())),
                enum_values: Some(vec![
                    "utf-8".to_string(),
                    "utf-16".to_string(),
                    "latin1".to_string(),
                ]),
            },
            SkillParameter {
                name: "limit_lines".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of lines to read".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "start_line".to_string(),
                param_type: "integer".to_string(),
                description: "Line number to start reading from (0-indexed)".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "text_read",
            "parameters": {
                "path": "notes.txt"
            }
        })
    }

    fn example_output(&self) -> String {
        "Line 1: Hello world\nLine 2: This is a text file".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let encoding = parameters
            .get("encoding")
            .and_then(|v| v.as_str())
            .unwrap_or("utf-8");
        let limit_lines = parameters
            .get("limit_lines")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let start_line = parameters
            .get("start_line")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("Text file not found: {}", path);
        }

        let content = read_file_content_with_encoding(&validated_path.to_string_lossy(), encoding)?;
        let lines: Vec<&str> = content.lines().collect();

        if start_line >= lines.len() {
            anyhow::bail!(
                "Start line {} exceeds total lines ({})",
                start_line,
                lines.len()
            );
        }

        let end_line = if let Some(limit) = limit_lines {
            (start_line + limit).min(lines.len())
        } else {
            lines.len()
        };

        let selected_lines = &lines[start_line..end_line];
        let mut output = String::new();

        for (i, line) in selected_lines.iter().enumerate() {
            output.push_str(&format!("Line {}: {}\n", start_line + i + 1, line));
        }

        if limit_lines.is_some() && end_line < lines.len() {
            output.push_str(&format!("... and {} more lines\n", lines.len() - end_line));
        }

        output.push_str(&format!("Total lines: {}", lines.len()));

        Ok(output)
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TextWriteSkill;

#[async_trait::async_trait]
impl Skill for TextWriteSkill {
    fn name(&self) -> &str {
        "text_write"
    }

    fn description(&self) -> &str {
        "Write text content to a file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to create, save, or append to a plain text file"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the text file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("output.txt".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "content".to_string(),
                param_type: "string".to_string(),
                description: "Text content to write".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello, World!".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "append".to_string(),
                param_type: "boolean".to_string(),
                description: "Append to existing file instead of overwriting".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "encoding".to_string(),
                param_type: "string".to_string(),
                description: "File encoding (utf-8, utf-16)".to_string(),
                required: false,
                default: Some(Value::String("utf-8".to_string())),
                example: Some(Value::String("utf-8".to_string())),
                enum_values: Some(vec!["utf-8".to_string(), "utf-16".to_string()]),
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "text_write",
            "parameters": {
                "path": "output.txt",
                "content": "Hello, World!"
            }
        })
    }

    fn example_output(&self) -> String {
        "Text written to: output.txt".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let content = parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;
        let append = parameters
            .get("append")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let encoding = parameters
            .get("encoding")
            .and_then(|v| v.as_str())
            .unwrap_or("utf-8");

        let validated_path = validate_path(path, None)?;
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy())?;
        }

        if append {
            write_file_content_with_encoding(
                &validated_path.to_string_lossy(),
                content,
                true,
                encoding,
            )?;
            Ok(format!("Content appended to text file: {}", path))
        } else {
            write_file_content_with_encoding(
                &validated_path.to_string_lossy(),
                content,
                false,
                encoding,
            )?;
            Ok(format!("Text written to: {}", path))
        }
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TextSearchSkill;

#[async_trait::async_trait]
impl Skill for TextSearchSkill {
    fn name(&self) -> &str {
        "text_search"
    }

    fn description(&self) -> &str {
        "Search for patterns in text files"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to search for text patterns, find lines containing specific words, or grep through files"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the text file to search".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("log.txt".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Search pattern (supports regex)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("error".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "case_sensitive".to_string(),
                param_type: "boolean".to_string(),
                description: "Case sensitive search".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "regex".to_string(),
                param_type: "boolean".to_string(),
                description: "Treat pattern as regular expression".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "text_search",
            "parameters": {
                "path": "log.txt",
                "pattern": "error"
            }
        })
    }

    fn example_output(&self) -> String {
        "Line 5: Error: connection failed\nLine 12: error: invalid input".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let pattern = parameters
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'pattern' parameter"))?;
        let case_sensitive = parameters
            .get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let use_regex = parameters
            .get("regex")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("Text file not found: {}", path);
        }
        let content = read_file_content(&validated_path.to_string_lossy())?;
        let lines: Vec<&str> = content.lines().collect();
        let mut matches = Vec::new();
        if use_regex {
            let regex_pattern = if case_sensitive {
                regex::Regex::new(pattern)?
            } else {
                regex::Regex::new(&format!("(?i){}", pattern))?
            };

            for (i, line) in lines.iter().enumerate() {
                if regex_pattern.is_match(line) {
                    matches.push((i + 1, *line));
                }
            }
        } else {
            let search_pattern = if case_sensitive {
                pattern.to_string()
            } else {
                pattern.to_lowercase()
            };
            for (i, line) in lines.iter().enumerate() {
                let check_line = if case_sensitive {
                    line.to_string()
                } else {
                    line.to_lowercase()
                };
                if check_line.contains(&search_pattern) {
                    matches.push((i + 1, *line));
                }
            }
        }
        if matches.is_empty() {
            Ok(format!("No matches found for pattern: {}", pattern))
        } else {
            let mut output = String::new();
            let matches_len = matches.len();
            for (line_num, line) in matches {
                output.push_str(&format!("Line {}: {}\n", line_num, line));
            }
            output.push_str(&format!("\nTotal matches: {}", matches_len));
            Ok(output)
        }
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        parameters
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pattern"))?;
        Ok(())
    }
}

fn read_file_content_with_encoding(path: &str, encoding: &str) -> Result<String> {
    use std::fs;

    let bytes = fs::read(path)?;

    match encoding {
        "utf-8" => Ok(String::from_utf8(bytes)?),
        "utf-16" => {
            let utf16_data: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16(&utf16_data)
                .map_err(|e| anyhow::anyhow!("UTF-16 decode error: {}", e))
        }
        "latin1" => Ok(bytes.iter().map(|&b| b as char).collect()),
        _ => anyhow::bail!("Unsupported encoding: {}", encoding),
    }
}

fn write_file_content_with_encoding(
    path: &str,
    content: &str,
    append: bool,
    encoding: &str,
) -> Result<()> {
    use std::fs::{File, OpenOptions};
    use std::io::Write;

    let bytes: Vec<u8> = match encoding {
        "utf-8" => content.as_bytes().to_vec(),
        "utf-16" => {
            let mut utf16 = Vec::new();
            for c in content.encode_utf16() {
                utf16.extend_from_slice(&c.to_le_bytes());
            }
            utf16
        }
        _ => anyhow::bail!("Unsupported encoding for write: {}", encoding),
    };

    let mut file = if append {
        OpenOptions::new().create(true).append(true).open(path)?
    } else {
        File::create(path)?
    };

    file.write_all(&bytes)?;
    Ok(())
}
