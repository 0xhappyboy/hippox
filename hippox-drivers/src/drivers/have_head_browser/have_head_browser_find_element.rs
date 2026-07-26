//! Browser find element skill
//!
//! This driver provides functionality to find elements on the current page
//! using CSS selectors and optionally extract their text content.
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
/// Driver for finding elements on the current page
#[derive(Debug)]
pub struct HaveHeadBrowserFindElementDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserFindElementDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_find_element"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Find an element on the current page by CSS selector"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to check if an element exists or get its properties before interacting"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "CSS selector of the element to find".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("#submit-button".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "get_text".to_string(),
                param_type: "boolean".to_string(),
                description: "Also return the element's text content (default: false)".to_string(),
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
            "action": "have_head_browser_find_element",
            "parameters": {
                "selector": ".result-title"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Element found: .result-title".to_string();
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
        debug!("Executing have_head_browser_find_element driver");
        let selector = parameters.get("selector").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing required parameter: selector");
            return crate::DriverError::missing_parameter("selector");
        })?;
        let get_text = parameters.get("get_text").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Finding element: {} (get_text: {})", selector, get_text);
        let tab = get_current_tab().map_err(|e| {
            debug!("Failed to get current tab: {}", e);
            return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
        })?;
        match tab.find_element(selector) {
            Ok(element) => {
                let mut result = format!("Element found: {}", selector);
                if get_text {
                    let js = format!(
                        r#"
                        (function() {{
                            const el = document.querySelector('{}');
                            return el ? el.innerText || el.textContent || '' : '';
                        }})()
                        "#,
                        selector
                    );
                    if let Ok(eval_result) = tab.evaluate(&js, false) {
                        if let Some(text) = eval_result.value {
                            let text_str = text.to_string();
                            if !text_str.is_empty() && text_str != "null" {
                                result.push_str(&format!("\nText: {}", text_str));
                            }
                        }
                    }
                }
                info!("Element found: {}", selector);
                return Ok(result);
            }
            Err(e) => {
                warn!("Element not found: {} - {}", selector, e);
                return Err(crate::DriverError::execution(format!("Element not found: {} - {}", selector, e)));
            }
        }
    }
}
