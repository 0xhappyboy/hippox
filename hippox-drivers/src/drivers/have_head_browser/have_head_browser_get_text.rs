//! Browser get text skill - extract text from elements
//!
//! This driver provides functionality to extract text content from elements
//! on the current page using CSS selectors.
use super::shared::*;
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for extracting text from elements
#[derive(Debug)]
pub struct HaveHeadBrowserGetTextDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserGetTextDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_get_text"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get text content from an element on the current page"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to extract text from elements like paragraphs, headings, or any element with text"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "CSS selector of the element".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("h1".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "all".to_string(),
                param_type: "boolean".to_string(),
                description: "Get text from all matching elements (default: false)".to_string(),
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
            "action": "have_head_browser_get_text",
            "parameters": {
                "selector": ".result-title"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Text: Example Result Title".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::HaveHeadBrowser;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing have_head_browser_get_text driver");
        let selector = parameters.get("selector").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing required parameter: selector");
            return crate::DriverError::missing_parameter("selector");
        })?;
        let get_all = parameters.get("all").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Getting text from: {} (all: {})", selector, get_all);
        let tab = get_current_tab().map_err(|e| {
            debug!("Failed to get current tab: {}", e);
            return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
        })?;
        if get_all {
            let js = format!(
                r#"
                (function() {{
                    const elements = document.querySelectorAll('{}');
                    return Array.from(elements).map(el => el.innerText || el.textContent || '');
                }})()
                "#,
                selector
            );
            match tab.evaluate(&js, false) {
                Ok(result) => {
                    if let Some(value) = result.value {
                        if let Some(arr) = value.as_array() {
                            let texts: Vec<String> = arr
                                .iter()
                                .enumerate()
                                .filter_map(|(i, v)| {
                                    let text = v.to_string();
                                    if !text.is_empty() && text != "null" { Some(format!("[{}]: {}", i, text)) } else { None }
                                })
                                .collect();
                            if texts.is_empty() {
                                info!("No text found in matching elements");
                                return Ok("No text found in matching elements".to_string());
                            } else {
                                info!("Found {} elements with text", texts.len());
                                return Ok(format!("Found {} elements:\n{}", texts.len(), texts.join("\n")));
                            }
                        } else {
                            return Ok(format!("Text: {}", value.to_string()));
                        }
                    } else {
                        info!("No text found");
                        return Ok("No text found".to_string());
                    }
                }
                Err(e) => {
                    warn!("Failed to get text: {}", e);
                    return Err(crate::DriverError::execution(format!("Failed to get text: {}", e)));
                }
            }
        } else {
            let js = format!(
                r#"
                (function() {{
                    const el = document.querySelector('{}');
                    return el ? (el.innerText || el.textContent || '') : '';
                }})()
                "#,
                selector
            );
            match tab.evaluate(&js, false) {
                Ok(result) => {
                    let text = result.value.map(|v| v.to_string()).unwrap_or_else(|| "".to_string());
                    if text.is_empty() || text == "null" {
                        info!("No text found for selector: {}", selector);
                        return Ok("No text found".to_string());
                    } else {
                        info!("Text extracted: {}", &text[..text.len().min(50)]);
                        return Ok(format!("Text: {}", text));
                    }
                }
                Err(e) => {
                    warn!("Element not found or failed to get text: {}", e);
                    return Err(crate::DriverError::execution(format!("Element not found or failed to get text: {}", e)));
                }
            }
        }
    }
}
