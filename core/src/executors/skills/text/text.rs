//! Text processing utilities for diff, sort, deduplicate, and filter operations.
//!
//! This module provides several skills for text manipulation:
//! - `TextDiffSkill`: Compare text differences between two strings
//! - `TextSortSkill`: Sort lines of text alphabetically or numerically
//! - `TextDeduplicateSkill`: Remove duplicate lines while preserving order
//! - `TextFilterSkill`: Filter lines by keyword or regex pattern

use crate::executors::types::{Skill, SkillParameter};
use anyhow::Result;
use serde_json::{Value, json};
use similar::{Algorithm, ChangeTag, TextDiff};
use std::collections::{HashMap, HashSet};

/// A skill for comparing text differences between two strings.
///
/// # Examples
/// ```
/// let result = text_diff.execute(&HashMap::from([
///     ("text1".to_string(), json!("Hello World")),
///     ("text2".to_string(), json!("Hello Rust")),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct TextDiffSkill;

#[async_trait::async_trait]
impl Skill for TextDiffSkill {
    fn name(&self) -> &str {
        "text_diff"
    }

    fn description(&self) -> &str {
        "Compare two texts and show the differences"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to compare configuration files, code changes, or document versions"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "text1".to_string(),
                param_type: "string".to_string(),
                description: "First text to compare".to_string(),
                required: true,
                default: None,
                example: Some(json!("original text")),
                enum_values: None,
            },
            SkillParameter {
                name: "text2".to_string(),
                param_type: "string".to_string(),
                description: "Second text to compare".to_string(),
                required: true,
                default: None,
                example: Some(json!("modified text")),
                enum_values: None,
            },
            SkillParameter {
                name: "unified_lines".to_string(),
                param_type: "integer".to_string(),
                description: "Number of context lines in unified diff (default: 3)".to_string(),
                required: false,
                default: Some(json!(3)),
                example: Some(json!(2)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "text_diff",
            "parameters": {
                "text1": "Hello World\nLine 2\nLine 3",
                "text2": "Hello Rust\nLine 2\nChanged Line"
            }
        })
    }

    fn example_output(&self) -> String {
        "--- text1\n+++ text2\n@@ -1,3 +1,3 @@\n-Hello World\n+Hello Rust\n Line 2\n-Line 3\n+Changed Line".to_string()
    }

    fn category(&self) -> &str {
        "text"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let text1 = parameters
            .get("text1")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text1"))?;
        let text2 = parameters
            .get("text2")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text2"))?;
        let diff = TextDiff::configure()
            .algorithm(Algorithm::Patience)
            .diff_lines(text1, text2);
        let mut result = Vec::new();
        for change in diff.iter_all_changes() {
            let prefix = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            result.push(format!(
                "{}{}",
                prefix,
                change.as_str().unwrap_or("").trim_end()
            ));
        }
        if result.is_empty() {
            Ok("No differences found".to_string())
        } else {
            Ok(result.join("\n"))
        }
    }
}

/// A skill for sorting lines of text.
///
/// # Examples
/// ```
/// let result = text_sort.execute(&HashMap::from([
///     ("text".to_string(), json!("banana\napple\ncherry")),
///     ("order".to_string(), json!("asc")),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct TextSortSkill;

#[async_trait::async_trait]
impl Skill for TextSortSkill {
    fn name(&self) -> &str {
        "text_sort"
    }

    fn description(&self) -> &str {
        "Sort lines of text alphabetically or numerically"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to organize data, prepare reports, or sort lists"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text to sort (lines separated by newline)".to_string(),
                required: true,
                default: None,
                example: Some(json!("line 3\nline 1\nline 2")),
                enum_values: None,
            },
            SkillParameter {
                name: "order".to_string(),
                param_type: "string".to_string(),
                description: "Sort order: asc (ascending) or desc (descending)".to_string(),
                required: false,
                default: Some(json!("asc")),
                example: Some(json!("desc")),
                enum_values: Some(vec!["asc".to_string(), "desc".to_string()]),
            },
            SkillParameter {
                name: "numeric".to_string(),
                param_type: "boolean".to_string(),
                description: "Sort numerically instead of lexicographically".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "reverse".to_string(),
                param_type: "boolean".to_string(),
                description: "Reverse the sort order".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "unique".to_string(),
                param_type: "boolean".to_string(),
                description: "Remove duplicates after sorting".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "text_sort",
            "parameters": {
                "text": "10\n2\n30\n4",
                "numeric": true
            }
        })
    }

    fn example_output(&self) -> String {
        "2\n4\n10\n30".to_string()
    }

    fn category(&self) -> &str {
        "text"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let text = parameters
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text"))?;
        let numeric = parameters
            .get("numeric")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let reverse = parameters
            .get("reverse")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let unique = parameters
            .get("unique")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let lines: Vec<&str> = text.lines().collect();
        let mut sorted_lines: Vec<String> = if numeric {
            let mut nums: Vec<f64> = lines
                .iter()
                .filter_map(|l| l.trim().parse::<f64>().ok())
                .collect();
            if reverse {
                nums.sort_by(|a, b| b.partial_cmp(a).unwrap());
            } else {
                nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
            }
            nums.into_iter().map(|n| n.to_string()).collect()
        } else {
            let mut strings: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            if reverse {
                strings.sort_by(|a, b| b.cmp(a));
            } else {
                strings.sort();
            }
            strings
        };
        if unique {
            let mut seen = HashSet::new();
            sorted_lines.retain(|line| seen.insert(line.clone()));
        }
        Ok(sorted_lines.join("\n"))
    }
}

/// A skill for removing duplicate lines from text.
///
/// # Examples
/// ```
/// let result = text_deduplicate.execute(&HashMap::from([
///     ("text".to_string(), json!("apple\nbanana\napple\ncherry\nbanana")),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct TextDeduplicateSkill;

#[async_trait::async_trait]
impl Skill for TextDeduplicateSkill {
    fn name(&self) -> &str {
        "text_deduplicate"
    }

    fn description(&self) -> &str {
        "Remove duplicate lines from text while preserving order"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you have duplicate entries in lists, logs, or data files"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text with potential duplicate lines".to_string(),
                required: true,
                default: None,
                example: Some(json!("line1\nline2\nline1\nline3")),
                enum_values: None,
            },
            SkillParameter {
                name: "case_sensitive".to_string(),
                param_type: "boolean".to_string(),
                description: "Treat lines with different case as different (default: true)"
                    .to_string(),
                required: false,
                default: Some(json!(true)),
                example: Some(json!(false)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "text_deduplicate",
            "parameters": {
                "text": "Red\nred\nBlue\nblue\nRed"
            }
        })
    }

    fn example_output(&self) -> String {
        "Red\nblue".to_string()
    }

    fn category(&self) -> &str {
        "text"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let text = parameters
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text"))?;
        let case_sensitive = parameters
            .get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for line in text.lines() {
            let key = if case_sensitive {
                line.to_string()
            } else {
                line.to_lowercase()
            };
            if !seen.contains(&key) {
                seen.insert(key);
                result.push(line);
            }
        }
        Ok(result.join("\n"))
    }
}

/// A skill for filtering lines by keyword or regex pattern.
///
/// # Examples
/// ```
/// let result = text_filter.execute(&HashMap::from([
///     ("text".to_string(), json!("error: file not found\ninfo: started\nwarning: timeout")),
///     ("pattern".to_string(), json!("error|warning")),
/// ])).await?;
/// ```
#[derive(Debug)]
pub struct TextFilterSkill;

#[async_trait::async_trait]
impl Skill for TextFilterSkill {
    fn name(&self) -> &str {
        "text_filter"
    }

    fn description(&self) -> &str {
        "Filter lines by keyword or regular expression pattern"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to extract specific lines from logs, search through text, or filter data"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text to filter".to_string(),
                required: true,
                default: None,
                example: Some(json!("log line 1\nlog line 2\nerror: something")),
                enum_values: None,
            },
            SkillParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Keyword or regex pattern to filter by".to_string(),
                required: true,
                default: None,
                example: Some(json!("error")),
                enum_values: None,
            },
            SkillParameter {
                name: "invert".to_string(),
                param_type: "boolean".to_string(),
                description: "If true, exclude matching lines (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "regex".to_string(),
                param_type: "boolean".to_string(),
                description: "Treat pattern as regex (default: false)".to_string(),
                required: false,
                default: Some(json!(false)),
                example: Some(json!(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "case_sensitive".to_string(),
                param_type: "boolean".to_string(),
                description: "Case-sensitive matching (default: true)".to_string(),
                required: false,
                default: Some(json!(true)),
                example: Some(json!(false)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "text_filter",
            "parameters": {
                "text": "ERROR: disk full\nINFO: started\nWARNING: low memory",
                "pattern": "ERROR|WARNING"
            }
        })
    }

    fn example_output(&self) -> String {
        "ERROR: disk full\nWARNING: low memory".to_string()
    }

    fn category(&self) -> &str {
        "text"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let text = parameters
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text"))?;
        let pattern = parameters
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pattern"))?;
        let invert = parameters
            .get("invert")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let use_regex = parameters
            .get("regex")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let case_sensitive = parameters
            .get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let mut filtered = Vec::new();
        if use_regex {
            let regex = if case_sensitive {
                regex::Regex::new(pattern)?
            } else {
                regex::Regex::new(&format!("(?i){}", pattern))?
            };
            for line in text.lines() {
                let matches = regex.is_match(line);
                if (!invert && matches) || (invert && !matches) {
                    filtered.push(line);
                }
            }
        } else {
            let pattern_lower = if !case_sensitive {
                pattern.to_lowercase()
            } else {
                pattern.to_string()
            };
            for line in text.lines() {
                let line_compare = if !case_sensitive {
                    line.to_lowercase()
                } else {
                    line.to_string()
                };
                let matches = line_compare.contains(&pattern_lower);
                if (!invert && matches) || (invert && !matches) {
                    filtered.push(line);
                }
            }
        }
        if filtered.is_empty() {
            Ok("No lines matched".to_string())
        } else {
            Ok(filtered.join("\n"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_text_diff() {
        let skill = TextDiffSkill;
        let mut params = HashMap::new();
        params.insert("text1".to_string(), json!("Hello World"));
        params.insert("text2".to_string(), json!("Hello Rust"));
        let result = skill.execute(&params).await.unwrap();
        assert!(result.contains("World") || result.contains("Rust"));
    }

    #[tokio::test]
    async fn test_text_sort() {
        let skill = TextSortSkill;
        let mut params = HashMap::new();
        params.insert("text".to_string(), json!("c\na\nb"));
        let result = skill.execute(&params).await.unwrap();
        assert_eq!(result, "a\nb\nc");
    }

    #[tokio::test]
    async fn test_text_deduplicate() {
        let skill = TextDeduplicateSkill;
        let mut params = HashMap::new();
        params.insert("text".to_string(), json!("a\nb\na\nc"));
        let result = skill.execute(&params).await.unwrap();
        assert_eq!(result, "a\nb\nc");
    }

    #[tokio::test]
    async fn test_text_filter() {
        let skill = TextFilterSkill;
        let mut params = HashMap::new();
        params.insert("text".to_string(), json!("error\ninfo\nerror\nwarning"));
        params.insert("pattern".to_string(), json!("error"));
        let result = skill.execute(&params).await.unwrap();
        assert_eq!(result, "error\nerror");
    }
}
