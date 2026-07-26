//! HTML parse driver
//!
//! This driver provides functionality to parse HTML content and extract
//! information like title, links, images, headings, and metadata.
use crate::common::html::parse_html;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for parsing HTML content
#[derive(Debug)]
pub struct HtmlParseDriver;
#[async_trait::async_trait]
impl Driver for HtmlParseDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "html_parse"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Parse HTML content and extract information like title, links, images, headings, and metadata"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to extract structured information from HTML content"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "html".to_string(),
                param_type: "string".to_string(),
                description: "HTML content to parse".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("<html><head><title>Example</title></head></html>".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "extract_all".to_string(),
                param_type: "boolean".to_string(),
                description: "Extract all elements (title, links, images, headings, paragraphs, metadata)".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "html_parse",
            "parameters": {
                "html": "<html><head><title>Example</title></head><body><p>Hello</p></body></html>"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Title: Example\nLinks: []\nImages: []\nHeadings: []\nParagraphs: ['Hello']".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing html_parse driver");
        let html = parameters.get("html").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("html"))?;
        let extract_all = parameters.get("extract_all").and_then(|v| v.as_bool()).unwrap_or(true);
        info!("HTML parse: length={}, extract_all={}", html.len(), extract_all);
        let result = parse_html(html, extract_all).map_err(|e| DriverError::execution(format!("Failed to parse HTML: {}", e)))?;
        info!(
            "HTML parse completed: title={:?}, links={}, images={}, headings={}, paragraphs={}",
            result.title,
            result.links.len(),
            result.images.len(),
            result.headings.len(),
            result.paragraphs.len()
        );
        let mut output = String::new();
        if let Some(title) = &result.title {
            output.push_str(&format!("Title: {}\n", title));
        }
        if extract_all {
            output.push_str(&format!("Links ({}): {}\n", result.links.len(), result.links.join(", ")));
            output.push_str(&format!("Images ({}): {}\n", result.images.len(), result.images.join(", ")));
            output.push_str(&format!("Headings ({}): {}\n", result.headings.len(), result.headings.join("; ")));
            output.push_str(&format!("Paragraphs ({}): {}\n", result.paragraphs.len(), result.paragraphs.join("; ")));
            if let Some(desc) = result.meta_description {
                output.push_str(&format!("Meta Description: {}\n", desc));
            }
            if let Some(keywords) = result.meta_keywords {
                output.push_str(&format!("Meta Keywords: {}\n", keywords));
            }
        }
        if output.is_empty() {
            output = "No content extracted".to_string();
            info!("No content extracted from HTML");
        }
        return Ok(output);
    }
}
