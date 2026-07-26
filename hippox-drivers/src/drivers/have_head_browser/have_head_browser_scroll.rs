//! Browser scroll skill
//!
//! This driver provides functionality to scroll the page to specified positions
//! or to specific elements.
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
/// Driver for scrolling the page
#[derive(Debug)]
pub struct HaveHeadBrowserScrollDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserScrollDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_scroll"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Scroll the page to a specified position or to an element"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to scroll the page to bring elements into view"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "Horizontal scroll position in pixels".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Vertical scroll position in pixels".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "CSS selector of element to scroll to (overrides x/y)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("#footer".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "behavior".to_string(),
                param_type: "string".to_string(),
                description: "Scroll behavior: auto or smooth (default: auto)".to_string(),
                required: false,
                default: Some(Value::String("auto".to_string())),
                example: Some(Value::String("smooth".to_string())),
                enum_values: Some(vec!["auto".to_string(), "smooth".to_string()]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_scroll",
            "parameters": {
                "y": 1000
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Scrolled to (0, 1000)".to_string();
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
        debug!("Executing have_head_browser_scroll driver");
        let tab = get_current_tab().map_err(|e| {
            debug!("Failed to get current tab: {}", e);
            return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
        })?;
        let behavior = parameters.get("behavior").and_then(|v| v.as_str()).unwrap_or("auto");
        if let Some(selector) = parameters.get("selector").and_then(|v| v.as_str()) {
            debug!("Scrolling to element: {} (behavior: {})", selector, behavior);
            let js = format!(
                r#"
                const element = document.querySelector('{}');
                if (element) {{
                    element.scrollIntoView({{ behavior: '{}' }});
                    true;
                }} else {{
                    false;
                }}
                "#,
                selector, behavior
            );
            let result = tab.evaluate(&js, false).map_err(|e| {
                warn!("Failed to scroll to element: {}", e);
                return crate::DriverError::execution(format!("Failed to scroll to element: {}", e));
            })?;
            let found = result.value.and_then(|v| v.as_bool()).unwrap_or(false);
            if found {
                info!("Scrolled to element: {}", selector);
                return Ok(format!("Scrolled to element: {}", selector));
            } else {
                warn!("Element not found: {}", selector);
                return Err(crate::DriverError::execution(format!("Element not found: {}", selector)));
            }
        } else {
            let x = parameters.get("x").and_then(|v| v.as_u64()).unwrap_or(0);
            let y = parameters.get("y").and_then(|v| v.as_u64()).unwrap_or(0);
            debug!("Scrolling to ({}, {}) with behavior: {}", x, y, behavior);
            let js = format!("window.scrollTo({{ left: {}, top: {}, behavior: '{}' }});", x, y, behavior);
            tab.evaluate(&js, false).map_err(|e| {
                warn!("Failed to scroll: {}", e);
                return crate::DriverError::execution(format!("Failed to scroll: {}", e));
            })?;
            info!("Scrolled to ({}, {})", x, y);
            return Ok(format!("Scrolled to ({}, {})", x, y));
        }
    }
}
