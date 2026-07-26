//! Text file driver module
//!
//! This module provides drivers for plain text file operations including
//! reading text files, writing text files, searching for patterns in text files,
//! and handling different text encodings.
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{DriverError, DriverResult, ensure_dir, file_exists, read_file_content, validate_path, write_file_content};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for reading plain text files
#[derive(Debug)]
pub struct TextReadDriver;
#[async_trait::async_trait]
impl Driver for TextReadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "text_read";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Read plain text file content";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to read a plain text file, view logs, or extract content from .txt files";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the text file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("notes.txt".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "encoding".to_string(),
                param_type: "string".to_string(),
                description: "File encoding (utf-8, utf-16, latin1)".to_string(),
                required: false,
                default: Some(Value::String("utf-8".to_string())),
                example: Some(Value::String("utf-8".to_string())),
                enum_values: Some(vec!["utf-8".to_string(), "utf-16".to_string(), "latin1".to_string()]),
            },
            DriverParameter {
                name: "limit_lines".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of lines to read".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "start_line".to_string(),
                param_type: "integer".to_string(),
                description: "Line number to start reading from (0-indexed)".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "text_read",
            "parameters": {
                "path": "notes.txt"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Line 1: Hello world\nLine 2: This is a text file".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Document;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing text_read driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let encoding = parameters.get("encoding").and_then(|v| v.as_str()).unwrap_or("utf-8");
        let limit_lines = parameters.get("limit_lines").and_then(|v| v.as_u64()).map(|v| v as usize);
        let start_line = parameters.get("start_line").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        debug!("Reading text file: {}, encoding: {}, start_line: {}, limit_lines: {:?}", path, encoding, start_line, limit_lines);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("Text file not found: {}", path)));
        }
        let content = read_file_content_with_encoding(&validated_path.to_string_lossy(), encoding)?;
        let lines: Vec<&str> = content.lines().collect();
        if start_line >= lines.len() {
            return Err(DriverError::execution(format!("Start line {} exceeds total lines ({})", start_line, lines.len())));
        }
        let end_line = if let Some(limit) = limit_lines { (start_line + limit).min(lines.len()) } else { lines.len() };
        let selected_lines = &lines[start_line..end_line];
        let mut output = String::new();
        for (i, line) in selected_lines.iter().enumerate() {
            output.push_str(&format!("Line {}: {}\n", start_line + i + 1, line));
        }
        if limit_lines.is_some() && end_line < lines.len() {
            output.push_str(&format!("... and {} more lines\n", lines.len() - end_line));
        }
        output.push_str(&format!("Total lines: {}", lines.len()));
        info!("Text read completed: {} ({} lines)", path, lines.len());
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Driver for writing text files
#[derive(Debug)]
pub struct TextWriteDriver;
#[async_trait::async_trait]
impl Driver for TextWriteDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "text_write";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Write text content to a file";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to create, save, or append to a plain text file";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the text file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("output.txt".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "content".to_string(),
                param_type: "string".to_string(),
                description: "Text content to write".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello, World!".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "append".to_string(),
                param_type: "boolean".to_string(),
                description: "Append to existing file instead of overwriting".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "encoding".to_string(),
                param_type: "string".to_string(),
                description: "File encoding (utf-8, utf-16)".to_string(),
                required: false,
                default: Some(Value::String("utf-8".to_string())),
                example: Some(Value::String("utf-8".to_string())),
                enum_values: Some(vec!["utf-8".to_string(), "utf-16".to_string()]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "text_write",
            "parameters": {
                "path": "output.txt",
                "content": "Hello, World!"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Text written to: output.txt".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Document;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing text_write driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let content = parameters.get("content").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("content"))?;
        let append = parameters.get("append").and_then(|v| v.as_bool()).unwrap_or(false);
        let encoding = parameters.get("encoding").and_then(|v| v.as_str()).unwrap_or("utf-8");
        debug!("Writing text file: {}, append: {}, encoding: {}", path, append, encoding);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to create directory: {}", e)))?;
        }
        if append {
            write_file_content_with_encoding(&validated_path.to_string_lossy(), content, true, encoding)?;
            info!("Text appended to: {}", path);
            return Ok(format!("Content appended to text file: {}", path));
        } else {
            write_file_content_with_encoding(&validated_path.to_string_lossy(), content, false, encoding)?;
            info!("Text written to: {}", path);
            return Ok(format!("Text written to: {}", path));
        }
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        parameters.get("content").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("content"))?;
        return Ok(());
    }
}
/// Driver for searching patterns in text files
#[derive(Debug)]
pub struct TextSearchDriver;
#[async_trait::async_trait]
impl Driver for TextSearchDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "text_search";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Search for patterns in text files";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to search for text patterns, find lines containing specific words, or grep through files";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the text file to search".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("log.txt".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Search pattern (supports regex)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("error".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "case_sensitive".to_string(),
                param_type: "boolean".to_string(),
                description: "Case sensitive search".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "regex".to_string(),
                param_type: "boolean".to_string(),
                description: "Treat pattern as regular expression".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "text_search",
            "parameters": {
                "path": "log.txt",
                "pattern": "error"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Line 5: Error: connection failed\nLine 12: error: invalid input".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Document;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing text_search driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let pattern = parameters.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("pattern"))?;
        let case_sensitive = parameters.get("case_sensitive").and_then(|v| v.as_bool()).unwrap_or(false);
        let use_regex = parameters.get("regex").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Searching text file: {}, pattern: {}, case_sensitive: {}, regex: {}", path, pattern, case_sensitive, use_regex);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("Text file not found: {}", path)));
        }
        let content =
            read_file_content(&validated_path.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to read file: {}", e)))?;
        let lines: Vec<&str> = content.lines().collect();
        let mut matches = Vec::new();
        if use_regex {
            let regex_pattern = if case_sensitive {
                regex::Regex::new(pattern).map_err(|e| DriverError::execution(format!("Invalid regex: {}", e)))?
            } else {
                regex::Regex::new(&format!("(?i){}", pattern)).map_err(|e| DriverError::execution(format!("Invalid regex: {}", e)))?
            };
            for (i, line) in lines.iter().enumerate() {
                if regex_pattern.is_match(line) {
                    matches.push((i + 1, *line));
                }
            }
        } else {
            let search_pattern = if case_sensitive { pattern.to_string() } else { pattern.to_lowercase() };
            for (i, line) in lines.iter().enumerate() {
                let check_line = if case_sensitive { line.to_string() } else { line.to_lowercase() };
                if check_line.contains(&search_pattern) {
                    matches.push((i + 1, *line));
                }
            }
        }
        if matches.is_empty() {
            info!("No matches found for pattern: {}", pattern);
            return Ok(format!("No matches found for pattern: {}", pattern));
        } else {
            let mut output = String::new();
            let matches_len = matches.len();
            for (line_num, line) in matches {
                output.push_str(&format!("Line {}: {}\n", line_num, line));
            }
            output.push_str(&format!("\nTotal matches: {}", matches_len));
            info!("Found {} matches for pattern: {}", matches_len, pattern);
            return Ok(output);
        }
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        parameters.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("pattern"))?;
        return Ok(());
    }
}
/// Reads file content with specified encoding
///
/// # Arguments
/// * `path` - File path
/// * `encoding` - Encoding name
///
/// # Returns
/// * `DriverResult<String>` - File content
fn read_file_content_with_encoding(path: &str, encoding: &str) -> DriverResult<String> {
    use std::fs;
    let bytes = fs::read(path).map_err(|e| DriverError::execution(format!("Failed to read file: {}", e)))?;
    match encoding {
        "utf-8" => String::from_utf8(bytes).map_err(|e| DriverError::execution(format!("UTF-8 decode error: {}", e))),
        "utf-16" => {
            let utf16_data: Vec<u16> = bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
            String::from_utf16(&utf16_data).map_err(|e| DriverError::execution(format!("UTF-16 decode error: {}", e)))
        }
        "latin1" => Ok(bytes.iter().map(|&b| b as char).collect()),
        _ => Err(DriverError::execution(format!("Unsupported encoding: {}", encoding))),
    }
}
/// Writes file content with specified encoding
///
/// # Arguments
/// * `path` - File path
/// * `content` - Content to write
/// * `append` - Whether to append to existing file
/// * `encoding` - Encoding name
///
/// # Returns
/// * `DriverResult<()>` - Success or error
fn write_file_content_with_encoding(path: &str, content: &str, append: bool, encoding: &str) -> DriverResult<()> {
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
        _ => {
            return Err(DriverError::execution(format!("Unsupported encoding for write: {}", encoding)));
        }
    };
    let mut file = if append {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| DriverError::execution(format!("Failed to open file for append: {}", e)))?
    } else {
        File::create(path).map_err(|e| DriverError::execution(format!("Failed to create file: {}", e)))?
    };
    file.write_all(&bytes).map_err(|e| DriverError::execution(format!("Failed to write file: {}", e)))?;
    return Ok(());
}
