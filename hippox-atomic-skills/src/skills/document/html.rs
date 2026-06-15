use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    ensure_dir, file_exists, read_file_content,
    types::{Skill, SkillParameter},
    validate_path, write_file_content,
};

#[derive(Debug)]
pub struct HtmlReadSkill;

#[async_trait::async_trait]
impl Skill for HtmlReadSkill {
    fn name(&self) -> &str {
        "html_read"
    }

    fn description(&self) -> &str {
        "Read and parse HTML file content"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read an HTML file, extract text content, or parse HTML structure"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the HTML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("index.html".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "extract_text".to_string(),
                param_type: "boolean".to_string(),
                description: "Extract only text content (strip HTML tags)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "CSS selector to extract specific elements".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("div.content".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "html_read",
            "parameters": {
                "path": "index.html"
            }
        })
    }

    fn example_output(&self) -> String {
        "<html><body><h1>Title</h1></body></html>".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let extract_text = parameters
            .get("extract_text")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let selector = parameters.get("selector").and_then(|v| v.as_str());

        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("HTML file not found: {}", path);
        }

        let content = read_file_content(&validated_path.to_string_lossy())?;

        if extract_text || selector.is_some() {
            use scraper::{Html, Selector};

            let document = Html::parse_document(&content);

            if let Some(sel_str) = selector {
                let selector = Selector::parse(sel_str)
                    .map_err(|e| anyhow::anyhow!("Invalid CSS selector: {}", e))?;

                let elements: Vec<String> = document
                    .select(&selector)
                    .map(|el| el.text().collect::<String>())
                    .collect();

                if elements.is_empty() {
                    Ok(format!("No elements found matching selector: {}", sel_str))
                } else {
                    let mut output = String::new();
                    for (i, text) in elements.iter().enumerate() {
                        output.push_str(&format!("Element {}: {}\n", i + 1, text));
                    }
                    output.push_str(&format!("\nTotal elements: {}", elements.len()));
                    Ok(output)
                }
            } else if extract_text {
                let text = document
                    .root_element()
                    .text()
                    .collect::<Vec<&str>>()
                    .join(" ");
                Ok(text)
            } else {
                Ok(content)
            }
        } else {
            Ok(content)
        }
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
pub struct HtmlWriteSkill;

#[async_trait::async_trait]
impl Skill for HtmlWriteSkill {
    fn name(&self) -> &str {
        "html_write"
    }

    fn description(&self) -> &str {
        "Write HTML content to a file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to create or save an HTML file"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the HTML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("output.html".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "content".to_string(),
                param_type: "string".to_string(),
                description: "HTML content to write".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("<html><body>Hello</body></html>".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "minify".to_string(),
                param_type: "boolean".to_string(),
                description: "Minify HTML (remove extra whitespace)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "html_write",
            "parameters": {
                "path": "output.html",
                "content": "<html><body>Hello</body></html>"
            }
        })
    }

    fn example_output(&self) -> String {
        "HTML written to: output.html".to_string()
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
        let minify = parameters
            .get("minify")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let validated_path = validate_path(path, None)?;
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy())?;
        }
        let final_content = if minify {
            minify_html(content)
        } else {
            content.to_string()
        };
        write_file_content(&validated_path.to_string_lossy(), &final_content, false)?;
        Ok(format!("HTML written to: {}", path))
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
pub struct HtmlValidateSkill;

#[async_trait::async_trait]
impl Skill for HtmlValidateSkill {
    fn name(&self) -> &str {
        "html_validate"
    }

    fn description(&self) -> &str {
        "Validate HTML syntax and structure"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to check if an HTML file has valid syntax"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the HTML file to validate".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("index.html".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "html_validate",
            "parameters": {
                "path": "index.html"
            }
        })
    }

    fn example_output(&self) -> String {
        "HTML is valid".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("HTML file not found: {}", path);
        }
        let content = read_file_content(&validated_path.to_string_lossy())?;
        use scraper::Html;
        let document = Html::parse_document(&content);
        let has_html = document
            .select(&scraper::Selector::parse("html").unwrap())
            .next()
            .is_some();
        let has_body = document
            .select(&scraper::Selector::parse("body").unwrap())
            .next()
            .is_some();
        let has_head = document
            .select(&scraper::Selector::parse("head").unwrap())
            .next()
            .is_some();
        let mut warnings = Vec::new();
        if !has_html {
            warnings.push("Missing <html> tag");
        }
        if !has_body {
            warnings.push("Missing <body> tag");
        }
        if !has_head {
            warnings.push("Missing <head> tag");
        }
        let mut output = String::from("✓ HTML parsed successfully\n");
        output.push_str(&format!("  Title: {}\n", get_title(&document)));
        if warnings.is_empty() {
            output.push_str("  Structure: Complete\n");
        } else {
            output.push_str("  Warnings:\n");
            for warning in warnings {
                output.push_str(&format!("    - {}\n", warning));
            }
        }
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

fn minify_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_quote = false;
    let mut quote_char = '\0';
    let mut prev_char = '\0';
    for c in html.chars() {
        if c == '"' || c == '\'' {
            if !in_quote {
                in_quote = true;
                quote_char = c;
            } else if c == quote_char && prev_char != '\\' {
                in_quote = false;
            }
        }
        if c == '<' && !in_quote {
            in_tag = true;
            if !result.is_empty() && result.ends_with(' ') {
                result.pop();
            }
            result.push(c);
        } else if c == '>' && in_tag {
            in_tag = false;
            result.push(c);
            if !result.ends_with('\n') {
                result.push('\n');
            }
        } else if in_tag || in_quote {
            result.push(c);
        } else if !c.is_whitespace() {
            result.push(c);
        } else if !result.is_empty() && !result.ends_with(' ') && !result.ends_with('\n') {
            result.push(' ');
        }
        prev_char = c;
    }
    result
}

fn get_title(document: &scraper::Html) -> String {
    if let Ok(selector) = scraper::Selector::parse("title") {
        if let Some(title_elem) = document.select(&selector).next() {
            return title_elem.text().collect::<String>();
        }
    }
    "No title found".to_string()
}
