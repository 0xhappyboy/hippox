//! Browser type driver - type text into input field
//!
//! This driver provides functionality to type text into input fields,
//! textareas, and other editable elements on the current page.
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
/// Driver for typing text into input fields
#[derive(Debug)]
pub struct HaveHeadBrowserTypeDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserTypeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_type"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Type text into an input field on the current page"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to fill input fields, textareas, or search boxes. First click the field if needed."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "CSS selector of the input element".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("#search-input".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text to type into the input field".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello World".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "clear_first".to_string(),
                param_type: "boolean".to_string(),
                description: "Clear existing text before typing (default: true)".to_string(),
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
            "action": "have_head_browser_type",
            "parameters": {
                "selector": "#search-input",
                "text": "Rust programming"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Typed 'Rust programming' into #search-input".to_string();
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
        debug!("Executing have_head_browser_type driver");
        let selector = parameters.get("selector").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing required parameter: selector");
            return crate::DriverError::missing_parameter("selector");
        })?;
        let text = parameters.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing required parameter: text");
            return crate::DriverError::missing_parameter("text");
        })?;
        let clear_first = parameters.get("clear_first").and_then(|v| v.as_bool()).unwrap_or(true);
        debug!("Typing text into: {} (clear_first: {})", selector, clear_first);
        let tab = get_current_tab().map_err(|e| {
            debug!("Failed to get current tab: {}", e);
            return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
        })?;
        let element = tab.find_element(selector).map_err(|e| {
            warn!("Element not found: '{}' - {}", selector, e);
            return crate::DriverError::execution(format!("Element not found: '{}' - {}", selector, e));
        })?;
        if clear_first {
            element.click().map_err(|e| {
                warn!("Failed to click element: {}", e);
                return crate::DriverError::execution(format!("Failed to click element: {}", e));
            })?;
        }
        let js = if clear_first {
            format!(
                r#"
                (function() {{
                    const el = document.querySelector('{}');
                    if (el) {{
                        el.value = '';
                        el.value = {};
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        return true;
                    }}
                    return false;
                }})()
                "#,
                selector,
                serde_json::to_string(text).map_err(|e| {
                    debug!("Failed to serialize text: {}", e);
                    return crate::DriverError::execution(format!("Failed to serialize text: {}", e));
                })?
            )
        } else {
            format!(
                r#"
                (function() {{
                    const el = document.querySelector('{}');
                    if (el) {{
                        el.value = {};
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        return true;
                    }}
                    return false;
                }})()
                "#,
                selector,
                serde_json::to_string(text).map_err(|e| {
                    debug!("Failed to serialize text: {}", e);
                    return crate::DriverError::execution(format!("Failed to serialize text: {}", e));
                })?
            )
        };
        let result = tab.evaluate(&js, false).map_err(|e| {
            warn!("Failed to type text: {}", e);
            return crate::DriverError::execution(format!("Failed to type text: {}", e));
        })?;
        if !result.value.and_then(|v| v.as_bool()).unwrap_or(false) {
            warn!("Element not found: {}", selector);
            return Err(crate::DriverError::execution(format!("Element not found: {}", selector)));
        }
        info!("Typed '{}' into {}", text, selector);
        return Ok(format!("Typed '{}' into {}", text, selector));
    }
}
