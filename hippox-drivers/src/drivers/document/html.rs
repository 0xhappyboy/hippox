//! HTML file driver module
//!
//! This module provides drivers for HTML file operations including
//! reading HTML files, extracting text content, parsing HTML structure,
//! writing HTML files, and validating HTML syntax.
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
/// Driver for reading HTML files
#[derive(Debug)]
pub struct HtmlReadDriver;
#[async_trait::async_trait]
impl Driver for HtmlReadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "html_read";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Read and parse HTML file content";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to read an HTML file, extract text content, or parse HTML structure";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the HTML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("index.html".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "extract_text".to_string(),
                param_type: "boolean".to_string(),
                description: "Extract only text content (strip HTML tags)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "CSS selector to extract specific elements".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("div.content".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "html_read",
            "parameters": {
                "path": "index.html"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "<html><body><h1>Title</h1></body></html>".to_string();
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
        debug!("Executing html_read driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let extract_text = parameters.get("extract_text").and_then(|v| v.as_bool()).unwrap_or(false);
        let selector = parameters.get("selector").and_then(|v| v.as_str());
        debug!("Reading HTML file: {}, extract_text: {}, selector: {:?}", path, extract_text, selector);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("HTML file not found: {}", path)));
        }
        let content =
            read_file_content(&validated_path.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to read file: {}", e)))?;
        if extract_text || selector.is_some() {
            use scraper::{Html, Selector};
            let document = Html::parse_document(&content);
            if let Some(sel_str) = selector {
                let selector = Selector::parse(sel_str).map_err(|e| DriverError::execution(format!("Invalid CSS selector: {}", e)))?;
                let elements: Vec<String> = document.select(&selector).map(|el| el.text().collect::<String>()).collect();
                if elements.is_empty() {
                    info!("No elements found matching selector: {}", sel_str);
                    return Ok(format!("No elements found matching selector: {}", sel_str));
                } else {
                    let mut output = String::new();
                    for (i, text) in elements.iter().enumerate() {
                        output.push_str(&format!("Element {}: {}\n", i + 1, text));
                    }
                    output.push_str(&format!("\nTotal elements: {}", elements.len()));
                    info!("Found {} elements matching selector: {}", elements.len(), sel_str);
                    return Ok(output);
                }
            } else if extract_text {
                let text = document.root_element().text().collect::<Vec<&str>>().join(" ");
                info!("Extracted text content, length: {} characters", text.len());
                return Ok(text);
            } else {
                return Ok(content);
            }
        } else {
            info!("HTML read completed, content length: {} bytes", content.len());
            return Ok(content);
        }
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Driver for writing HTML files
#[derive(Debug)]
pub struct HtmlWriteDriver;
#[async_trait::async_trait]
impl Driver for HtmlWriteDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "html_write";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Write HTML content to a file";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to create or save an HTML file";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the HTML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("output.html".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "content".to_string(),
                param_type: "string".to_string(),
                description: "HTML content to write".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("<html><body>Hello</body></html>".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "minify".to_string(),
                param_type: "boolean".to_string(),
                description: "Minify HTML (remove extra whitespace)".to_string(),
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
            "action": "html_write",
            "parameters": {
                "path": "output.html",
                "content": "<html><body>Hello</body></html>"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "HTML written to: output.html".to_string();
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
        debug!("Executing html_write driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let content = parameters.get("content").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("content"))?;
        let minify = parameters.get("minify").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Writing HTML file: {}, minify: {}", path, minify);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to create directory: {}", e)))?;
        }
        let final_content = if minify { minify_html(content) } else { content.to_string() };
        write_file_content(&validated_path.to_string_lossy(), &final_content, false)
            .map_err(|e| DriverError::execution(format!("Failed to write file: {}", e)))?;
        info!("HTML written to: {}", path);
        return Ok(format!("HTML written to: {}", path));
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        parameters.get("content").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("content"))?;
        return Ok(());
    }
}
/// Driver for validating HTML syntax
#[derive(Debug)]
pub struct HtmlValidateDriver;
#[async_trait::async_trait]
impl Driver for HtmlValidateDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "html_validate";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Validate HTML syntax and structure";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to check if an HTML file has valid syntax";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the HTML file to validate".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("index.html".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "html_validate",
            "parameters": {
                "path": "index.html"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "HTML is valid".to_string();
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
        debug!("Executing html_validate driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        debug!("Validating HTML file: {}", path);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("HTML file not found: {}", path)));
        }
        let content =
            read_file_content(&validated_path.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to read file: {}", e)))?;
        use scraper::Html;
        let document = Html::parse_document(&content);
        let has_html = document.select(&scraper::Selector::parse("html").unwrap()).next().is_some();
        let has_body = document.select(&scraper::Selector::parse("body").unwrap()).next().is_some();
        let has_head = document.select(&scraper::Selector::parse("head").unwrap()).next().is_some();
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
        let mut output = String::from("HTML parsed successfully\n");
        output.push_str(&format!("  Title: {}\n", get_title(&document)));
        if warnings.is_empty() {
            output.push_str("  Structure: Complete\n");
            info!("HTML is valid: {}", path);
        } else {
            output.push_str("  Warnings:\n");
            for warning in warnings {
                output.push_str(&format!("    - {}\n", warning));
            }
            info!("HTML validation completed with warnings: {}", path);
        }
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Minifies HTML content by removing unnecessary whitespace
///
/// # Arguments
/// * `html` - HTML content to minify
///
/// # Returns
/// * `String` - Minified HTML content
fn minify_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_quote = false;
    let mut quote_char = '\0';
    let mut prev_char = '\0';
    for c in html.chars() {
        // Track if we're inside quotes
        if c == '"' || c == '\'' {
            if !in_quote {
                in_quote = true;
                quote_char = c;
            } else if c == quote_char && prev_char != '\\' {
                in_quote = false;
            }
        }
        // Handle HTML tags
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
    return result;
}
/// Extracts the title from an HTML document
///
/// # Arguments
/// * `document` - Scraper HTML document
///
/// # Returns
/// * `String` - Document title or default message
fn get_title(document: &scraper::Html) -> String {
    if let Ok(selector) = scraper::Selector::parse("title") {
        if let Some(title_elem) = document.select(&selector).next() {
            return title_elem.text().collect::<String>();
        }
    }
    return "No title found".to_string();
}
