use crate::common::html::parse_html;
use crate::types::{Driver, DriverParameter};
use crate::{DriverCallback, DriverCategory, DriverContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct HtmlParseDriver;

#[async_trait::async_trait]
impl Driver for HtmlParseDriver {
    fn name(&self) -> &str {
        "html_parse"
    }

    fn description(&self) -> &str {
        "Parse HTML content and extract information like title, links, images, headings, and metadata"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to extract structured information from HTML content"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "html".to_string(),
                param_type: "string".to_string(),
                description: "HTML content to parse".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "<html><head><title>Example</title></head></html>".to_string(),
                )),
                enum_values: None,
            },
            DriverParameter {
                name: "extract_all".to_string(),
                param_type: "boolean".to_string(),
                description:
                    "Extract all elements (title, links, images, headings, paragraphs, metadata)"
                        .to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "html_parse",
            "parameters": {
                "html": "<html><head><title>Example</title></head><body><p>Hello</p></body></html>"
            }
        })
    }

    fn example_output(&self) -> String {
        "Title: Example\nLinks: []\nImages: []\nHeadings: []\nParagraphs: ['Hello']".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Network
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.driver_name())
            .map(String::from);
        let cb = callback;

        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Starting HTML parse".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }

        let html = parameters
            .get("html")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'html' parameter"))?;

        if let Some(cb) = cb {
            let preview = if html.len() > 100 {
                format!("{}...", &html[..100])
            } else {
                html.to_string()
            };
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("HTML length: {} bytes", html.len())),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }

        let extract_all = parameters
            .get("extract_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Extract all: {}", extract_all)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }

        let result = parse_html(html, extract_all)?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Parsing completed".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(80), None);
        }

        let mut output = String::new();
        if let Some(title) = &result.title {
            output.push_str(&format!("Title: {}\n", title));
        }
        if extract_all {
            output.push_str(&format!(
                "Links ({}): {}\n",
                result.links.len(),
                result.links.join(", ")
            ));
            output.push_str(&format!(
                "Images ({}): {}\n",
                result.images.len(),
                result.images.join(", ")
            ));
            output.push_str(&format!(
                "Headings ({}): {}\n",
                result.headings.len(),
                result.headings.join("; ")
            ));
            output.push_str(&format!(
                "Paragraphs ({}): {}\n",
                result.paragraphs.len(),
                result.paragraphs.join("; ")
            ));
            if let Some(desc) = result.meta_description {
                output.push_str(&format!("Meta Description: {}\n", desc));
            }
            if let Some(keywords) = result.meta_keywords {
                output.push_str(&format!("Meta Keywords: {}\n", keywords));
            }
        }

        if output.is_empty() {
            output = "No content extracted".to_string();
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Result size: {} chars", output.len())),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                driver_index,
                Some("html_parse".to_string()),
                Some(output.clone()),
            );
        }

        Ok(output)
    }
}
