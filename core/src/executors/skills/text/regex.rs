//! Regular expression utilities for pattern matching, extraction, and replacement.
//!
//! This module provides several skills for working with regular expressions:
//! - `RegexMatchSkill`: Check if a pattern matches a string
//! - `RegexFindSkill`: Find all matches of a pattern in a string
//! - `RegexReplaceSkill`: Replace pattern matches with a replacement string
//! - `RegexExtractSkill`: Extract capture groups from matches

use crate::executors::types::{Skill, SkillParameter};
use anyhow::Result;
use regex::Regex;
use serde_json::{Value, json};
use std::collections::HashMap;

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
pub struct RegexMatchSkill;

#[async_trait::async_trait]
impl Skill for RegexMatchSkill {
    fn name(&self) -> &str {
        "regex_match"
    }

    fn description(&self) -> &str {
        "Check if a regular expression pattern matches a string"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to validate string format, check if text matches a pattern, or perform pattern-based filtering"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Regular expression pattern to match".to_string(),
                required: true,
                default: None,
                example: Some(json!(r"^[a-zA-Z]+$")),
                enum_values: None,
            },
            SkillParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text to test against the pattern".to_string(),
                required: true,
                default: None,
                example: Some(json!("HelloWorld")),
                enum_values: None,
            },
            SkillParameter {
                name: "case_insensitive".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to ignore case (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "regex_match",
            "parameters": {
                "pattern": r"^\d{4}-\d{2}-\d{2}$",
                "text": "2024-01-15"
            }
        })
    }

    fn example_output(&self) -> String {
        "Pattern matches: true".to_string()
    }

    fn category(&self) -> &str {
        "text"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let pattern = parameters
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pattern"))?;
        let text = parameters
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text"))?;
        let case_insensitive = parameters
            .get("case_insensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let regex = if case_insensitive {
            Regex::new(&format!("(?i){}", pattern))?
        } else {
            Regex::new(pattern)?
        };
        let is_match = regex.is_match(text);
        Ok(format!("Pattern matches: {}", is_match))
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
pub struct RegexFindSkill;

#[async_trait::async_trait]
impl Skill for RegexFindSkill {
    fn name(&self) -> &str {
        "regex_find"
    }

    fn description(&self) -> &str {
        "Find all matches of a regular expression pattern in a string"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to extract all occurrences of a pattern from text"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Regular expression pattern to find".to_string(),
                required: true,
                default: None,
                example: Some(json!(r"\d+")),
                enum_values: None,
            },
            SkillParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text to search".to_string(),
                required: true,
                default: None,
                example: Some(json!("There are 42 apples and 7 oranges")),
                enum_values: None,
            },
            SkillParameter {
                name: "case_insensitive".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to ignore case (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "regex_find",
            "parameters": {
                "pattern": r"\b[A-Z][a-z]+\b",
                "text": "Hello World from Rust"
            }
        })
    }

    fn example_output(&self) -> String {
        "Found matches:\n  - Hello\n  - World\n  - Rust".to_string()
    }

    fn category(&self) -> &str {
        "text"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let pattern = parameters
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pattern"))?;
        let text = parameters
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text"))?;
        let case_insensitive = parameters
            .get("case_insensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let regex = if case_insensitive {
            Regex::new(&format!("(?i){}", pattern))?
        } else {
            Regex::new(pattern)?
        };
        let matches: Vec<&str> = regex.find_iter(text).map(|m| m.as_str()).collect();
        if matches.is_empty() {
            Ok("No matches found".to_string())
        } else {
            let result = format!("Found {} match(es):\n  {}", matches.len(), matches.join("\n  "));
            Ok(result)
        }
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
pub struct RegexReplaceSkill;

#[async_trait::async_trait]
impl Skill for RegexReplaceSkill {
    fn name(&self) -> &str {
        "regex_replace"
    }

    fn description(&self) -> &str {
        "Replace all matches of a regular expression pattern with a replacement string"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to redact sensitive information, format text, or perform search-and-replace operations"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Regular expression pattern to replace".to_string(),
                required: true,
                default: None,
                example: Some(json!(r"\s+")),
                enum_values: None,
            },
            SkillParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Original text".to_string(),
                required: true,
                default: None,
                example: Some(json!("Hello   World")),
                enum_values: None,
            },
            SkillParameter {
                name: "replacement".to_string(),
                param_type: "string".to_string(),
                description: "Replacement string (can use $1, $2 for capture groups)".to_string(),
                required: true,
                default: None,
                example: Some(json!(" ")),
                enum_values: None,
            },
            SkillParameter {
                name: "case_insensitive".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether to ignore case (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "regex_replace",
            "parameters": {
                "pattern": r"\b(\d{3})-(\d{4})\b",
                "text": "Call 555-1234 for support",
                "replacement": "[$1-$2]"
            }
        })
    }

    fn example_output(&self) -> String {
        "Result: Call [555-1234] for support".to_string()
    }

    fn category(&self) -> &str {
        "text"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let pattern = parameters
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pattern"))?;
        let text = parameters
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text"))?;
        let replacement = parameters
            .get("replacement")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: replacement"))?;
        let case_insensitive = parameters
            .get("case_insensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let regex = if case_insensitive {
            Regex::new(&format!("(?i){}", pattern))?
        } else {
            Regex::new(pattern)?
        };
        let result = regex.replace_all(text, replacement);
        Ok(format!("Result: {}", result))
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
pub struct RegexExtractSkill;

#[async_trait::async_trait]
impl Skill for RegexExtractSkill {
    fn name(&self) -> &str {
        "regex_extract"
    }

    fn description(&self) -> &str {
        "Extract capture groups from regular expression matches"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to extract specific parts of text like dates, IDs, or structured data"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Regular expression with capture groups (using parentheses)".to_string(),
                required: true,
                default: None,
                example: Some(json!(r"(\w+)@(\w+\.\w+)")),
                enum_values: None,
            },
            SkillParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text to extract from".to_string(),
                required: true,
                default: None,
                example: Some(json!("user@example.com")),
                enum_values: None,
            },
            SkillParameter {
                name: "first_only".to_string(),
                param_type: "boolean".to_string(),
                description: "Only return the first match (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "regex_extract",
            "parameters": {
                "pattern": r"(\d{2})/(\d{2})/(\d{4})",
                "text": "Today is 12/25/2024"
            }
        })
    }

    fn example_output(&self) -> String {
        "Extracted groups:\nMatch 1:\n  Group 1: 12\n  Group 2: 25\n  Group 3: 2024".to_string()
    }

    fn category(&self) -> &str {
        "text"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let pattern = parameters
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pattern"))?;
        let text = parameters
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text"))?;
        let first_only = parameters
            .get("first_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let regex = Regex::new(pattern)?;
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
        Ok(output.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_regex_match() {
        let skill = RegexMatchSkill;
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), json!(r"^\d+$"));
        params.insert("text".to_string(), json!("12345"));

        let result = skill.execute(&params).await.unwrap();
        assert!(result.contains("true"));
    }

    #[tokio::test]
    async fn test_regex_find() {
        let skill = RegexFindSkill;
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), json!(r"\d+"));
        params.insert("text".to_string(), json!("42 and 100"));

        let result = skill.execute(&params).await.unwrap();
        assert!(result.contains("42"));
        assert!(result.contains("100"));
    }

    #[tokio::test]
    async fn test_regex_replace() {
        let skill = RegexReplaceSkill;
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), json!(r"\d+"));
        params.insert("text".to_string(), json!("ID: 12345"));
        params.insert("replacement".to_string(), json!("[HIDDEN]"));

        let result = skill.execute(&params).await.unwrap();
        assert!(result.contains("ID: [HIDDEN]"));
    }

    #[tokio::test]
    async fn test_regex_extract() {
        let skill = RegexExtractSkill;
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), json!(r"(\w+)-(\d+)"));
        params.insert("text".to_string(), json!("item-42"));

        let result = skill.execute(&params).await.unwrap();
        assert!(result.contains("item"));
        assert!(result.contains("42"));
    }
}