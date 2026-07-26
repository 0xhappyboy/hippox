//! Regular expression utilities for pattern matching, extraction, and replacement.
//!
//! This module provides several skills for working with regular expressions:
//! - `RegexMatchDriver`: Check if a pattern matches a string
//! - `RegexFindDriver`: Find all matches of a pattern in a string
//! - `RegexReplaceDriver`: Replace pattern matches with a replacement string
//! - `RegexExtractDriver`: Extract capture groups from matches
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use regex::Regex;
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// A skill for checking if a pattern matches a string.
///
/// # Examples
/// ```
/// let result = regex_match.execute(&HashMap::from([
///     ("pattern".to_string(), json!(r"^\d{3}-\d{4}$")),
///     ("text".to_string(), json!("123-4567")),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct RegexMatchDriver;
#[async_trait::async_trait]
impl Driver for RegexMatchDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "regex_match";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Check if a regular expression pattern matches a string";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when you need to validate string format, check if text matches a pattern, or perform pattern-based filtering";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Regular expression pattern to match".to_string(),
                required: true,
                default: None,
                example: Some(json!(r"^[a-zA-Z]+$")),
                enum_values: None,
            },
            DriverParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text to test against the pattern".to_string(),
                required: true,
                default: None,
                example: Some(json!("HelloWorld")),
                enum_values: None,
            },
            DriverParameter {
                name: "case_insensitive".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to ignore case (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "regex_match",
            "parameters": {
                "pattern": r"^\d{4}-\d{2}-\d{2}$",
                "text": "2024-01-15"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Pattern matches: true".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Text;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing regex_match driver");
        let pattern = parameters.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("pattern"))?;
        let text = parameters.get("text").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("text"))?;
        let case_insensitive = parameters.get("case_insensitive").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Checking if pattern '{}' matches text", pattern);
        let regex = if case_insensitive {
            Regex::new(&format!("(?i){}", pattern)).map_err(|e| DriverError::execution(format!("Invalid regex pattern: {}", e)))?
        } else {
            Regex::new(pattern).map_err(|e| DriverError::execution(format!("Invalid regex pattern: {}", e)))?
        };
        let is_match = regex.is_match(text);
        let result = format!("Pattern matches: {}", is_match);
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("pattern"))?;
        parameters.get("text").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("text"))?;
        return Ok(());
    }
}
/// A skill for finding all matches of a pattern in a string.
///
/// # Examples
/// ```
/// let result = regex_find.execute(&HashMap::from([
///     ("pattern".to_string(), json!(r"\b\w+@\w+\.\w+\b")),
///     ("text".to_string(), json!("Contact: user@example.com or admin@test.org")),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct RegexFindDriver;
#[async_trait::async_trait]
impl Driver for RegexFindDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "regex_find";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Find all matches of a regular expression pattern in a string";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when you need to extract all occurrences of a pattern from text";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Regular expression pattern to find".to_string(),
                required: true,
                default: None,
                example: Some(json!(r"\d+")),
                enum_values: None,
            },
            DriverParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text to search".to_string(),
                required: true,
                default: None,
                example: Some(json!("There are 42 apples and 7 oranges")),
                enum_values: None,
            },
            DriverParameter {
                name: "case_insensitive".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to ignore case (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "regex_find",
            "parameters": {
                "pattern": r"\b[A-Z][a-z]+\b",
                "text": "Hello World from Rust"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Found matches:\n  - Hello\n  - World\n  - Rust".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Text;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing regex_find driver");
        let pattern = parameters.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("pattern"))?;
        let text = parameters.get("text").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("text"))?;
        let case_insensitive = parameters.get("case_insensitive").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Finding pattern '{}' in text", pattern);
        let regex = if case_insensitive {
            Regex::new(&format!("(?i){}", pattern)).map_err(|e| DriverError::execution(format!("Invalid regex pattern: {}", e)))?
        } else {
            Regex::new(pattern).map_err(|e| DriverError::execution(format!("Invalid regex pattern: {}", e)))?
        };
        let matches: Vec<&str> = regex.find_iter(text).map(|m| m.as_str()).collect();
        let result = if matches.is_empty() {
            "No matches found".to_string()
        } else {
            format!("Found {} match(es):\n  {}", matches.len(), matches.join("\n  "))
        };
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("pattern"))?;
        parameters.get("text").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("text"))?;
        return Ok(());
    }
}
/// A skill for replacing pattern matches with a replacement string.
///
/// # Examples
/// ```
/// let result = regex_replace.execute(&HashMap::from([
///     ("pattern".to_string(), json!(r"\d+")),
///     ("text".to_string(), json!("ID: 12345")),
///     ("replacement".to_string(), json!("[REDACTED]")),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct RegexReplaceDriver;
#[async_trait::async_trait]
impl Driver for RegexReplaceDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "regex_replace";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Replace all matches of a regular expression pattern with a replacement string";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when you need to redact sensitive information, format text, or perform search-and-replace operations";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Regular expression pattern to replace".to_string(),
                required: true,
                default: None,
                example: Some(json!(r"\s+")),
                enum_values: None,
            },
            DriverParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Original text".to_string(),
                required: true,
                default: None,
                example: Some(json!("Hello   World")),
                enum_values: None,
            },
            DriverParameter {
                name: "replacement".to_string(),
                param_type: "string".to_string(),
                description: "Replacement string (can use $1, $2 for capture groups)".to_string(),
                required: true,
                default: None,
                example: Some(json!(" ")),
                enum_values: None,
            },
            DriverParameter {
                name: "case_insensitive".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to ignore case (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "regex_replace",
            "parameters": {
                "pattern": r"\b(\d{3})-(\d{4})\b",
                "text": "Call 555-1234 for support",
                "replacement": "[$1-$2]"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Result: Call [555-1234] for support".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Text;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing regex_replace driver");
        let pattern = parameters.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("pattern"))?;
        let text = parameters.get("text").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("text"))?;
        let replacement = parameters.get("replacement").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("replacement"))?;
        let case_insensitive = parameters.get("case_insensitive").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Replacing pattern '{}' with '{}'", pattern, replacement);
        let regex = if case_insensitive {
            Regex::new(&format!("(?i){}", pattern)).map_err(|e| DriverError::execution(format!("Invalid regex pattern: {}", e)))?
        } else {
            Regex::new(pattern).map_err(|e| DriverError::execution(format!("Invalid regex pattern: {}", e)))?
        };
        let result = regex.replace_all(text, replacement);
        let result_str = format!("Result: {}", result);
        info!("{}", result_str);
        return Ok(result_str);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("pattern"))?;
        parameters.get("text").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("text"))?;
        parameters.get("replacement").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("replacement"))?;
        return Ok(());
    }
}
/// A skill for extracting capture groups from regex matches.
///
/// # Examples
/// ```
/// let result = regex_extract.execute(&HashMap::from([
///     ("pattern".to_string(), json!(r"(\d{4})-(\d{2})-(\d{2})")),
///     ("text".to_string(), json!("Date: 2024-01-15")),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct RegexExtractDriver;
#[async_trait::async_trait]
impl Driver for RegexExtractDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "regex_extract";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Extract capture groups from regular expression matches";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when you need to extract specific parts of text like dates, IDs, or structured data";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Regular expression with capture groups (using parentheses)".to_string(),
                required: true,
                default: None,
                example: Some(json!(r"(\w+)@(\w+\.\w+)")),
                enum_values: None,
            },
            DriverParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text to extract from".to_string(),
                required: true,
                default: None,
                example: Some(json!("user@example.com")),
                enum_values: None,
            },
            DriverParameter {
                name: "first_only".to_string(),
                param_type: "boolean".to_string(),
                description: "Only return the first match (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "regex_extract",
            "parameters": {
                "pattern": r"(\d{2})/(\d{2})/(\d{4})",
                "text": "Today is 12/25/2024"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Extracted groups:\nMatch 1:\n  Group 1: 12\n  Group 2: 25\n  Group 3: 2024".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Text;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing regex_extract driver");
        let pattern = parameters.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("pattern"))?;
        let text = parameters.get("text").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("text"))?;
        let first_only = parameters.get("first_only").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Extracting groups from pattern '{}'", pattern);
        let regex = Regex::new(pattern).map_err(|e| DriverError::execution(format!("Invalid regex pattern: {}", e)))?;
        let mut output = Vec::new();
        if first_only {
            if let Some(caps) = regex.captures(text) {
                output.push("Extracted groups:".to_string());
                for (i, cap) in caps.iter().enumerate() {
                    if let Some(m) = cap {
                        output.push(format!("  Group {}: {}", i, m.as_str()));
                    }
                }
            } else {
                return Ok("No matches found".to_string());
            }
        } else {
            let all_captures: Vec<regex::Captures> = regex.captures_iter(text).collect();
            if all_captures.is_empty() {
                return Ok("No matches found".to_string());
            }
            output.push(format!("Extracted groups ({} match(es)):", all_captures.len()));
            for (match_idx, caps) in all_captures.iter().enumerate() {
                output.push(format!("Match {}:", match_idx + 1));
                for (group_idx, cap) in caps.iter().enumerate() {
                    if let Some(m) = cap {
                        output.push(format!("  Group {}: {}", group_idx, m.as_str()));
                    }
                }
            }
        }
        let result = output.join("\n");
        info!("Extraction completed");
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("pattern"))?;
        parameters.get("text").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("text"))?;
        return Ok(());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_regex_match() {
        let skill = RegexMatchDriver;
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), json!(r"^\d+$"));
        params.insert("text".to_string(), json!("12345"));
        let result = skill.execute(&params, None, None).await.unwrap();
        assert!(result.contains("true"));
    }
    #[tokio::test]
    async fn test_regex_find() {
        let skill = RegexFindDriver;
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), json!(r"\d+"));
        params.insert("text".to_string(), json!("42 and 100"));
        let result = skill.execute(&params, None, None).await.unwrap();
        assert!(result.contains("42"));
        assert!(result.contains("100"));
    }
    #[tokio::test]
    async fn test_regex_replace() {
        let skill = RegexReplaceDriver;
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), json!(r"\d+"));
        params.insert("text".to_string(), json!("ID: 12345"));
        params.insert("replacement".to_string(), json!("[HIDDEN]"));
        let result = skill.execute(&params, None, None).await.unwrap();
        assert!(result.contains("ID: [HIDDEN]"));
    }
    #[tokio::test]
    async fn test_regex_extract() {
        let skill = RegexExtractDriver;
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), json!(r"(\w+)-(\d+)"));
        params.insert("text".to_string(), json!("item-42"));
        let result = skill.execute(&params, None, None).await.unwrap();
        assert!(result.contains("item"));
        assert!(result.contains("42"));
    }
}
