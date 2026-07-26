//! Browser click skill - click element by selector
//!
//! This driver provides functionality to click elements on the current page
//! using CSS selectors.
use super::shared::*;
use crate::{DriverCallback, DriverContext, DriverResult};
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter, },
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for clicking elements on the current page
#[derive(Debug)]
pub struct HaveHeadBrowserClickDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserClickDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_click"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Click an element on the current page by CSS selector"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to click buttons, links, or any clickable element. The browser window is visible so user can watch."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "CSS selector of the element to click".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("#submit-button".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "wait_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Milliseconds to wait after click (default: 1000)".to_string(),
                required: false,
                default: Some(Value::Number(1000.into())),
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_click",
            "parameters": {
                "selector": "#search-button"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Clicked element: #search-button".to_string();
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
        debug!("Executing have_head_browser_click driver");
        let selector = parameters.get("selector").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing required parameter: selector");
            return crate::DriverError::missing_parameter("selector");
        })?;
        let wait_ms = parameters.get("wait_ms").and_then(|v| v.as_u64()).unwrap_or(1000);
        debug!("Clicking element: {} (wait_ms: {})", selector, wait_ms);
        let tab = get_current_tab().map_err(|e| {
            debug!("Failed to get current tab: {}", e);
            return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
        })?;
        let element = tab.find_element(selector).map_err(|e| {
            warn!("Element not found: '{}' - {}", selector, e);
            return crate::DriverError::execution(format!("Element not found: '{}' - {}", selector, e));
        })?;
        element.click().map_err(|e| {
            warn!("Failed to click element '{}': {}", selector, e);
            return crate::DriverError::execution(format!("Failed to click element '{}': {}", selector, e));
        })?;
        wait_for_stable(&tab, wait_ms).await;
        info!("Clicked element: {}", selector);
        return Ok(format!("Clicked element: {}", selector));
    }
}
