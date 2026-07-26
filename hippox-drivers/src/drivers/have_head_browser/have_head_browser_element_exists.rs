//! Browser element exists check skill
//!
//! This driver provides functionality to check if an element exists
//! on the current page using a CSS selector.
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
use tracing::{debug, info};
/// Driver for checking if an element exists
#[derive(Debug)]
pub struct HaveHeadBrowserElementExistsDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserElementExistsDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_element_exists"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Check if an element exists on the current page"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to verify if an element is present before interacting with it"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "selector".to_string(),
            param_type: "string".to_string(),
            description: "CSS selector to check".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("#submit-button".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_element_exists",
            "parameters": {
                "selector": ".loading-spinner"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Element exists: true".to_string();
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
        debug!("Executing have_head_browser_element_exists driver");
        let selector = parameters.get("selector").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing required parameter: selector");
            return crate::DriverError::missing_parameter("selector");
        })?;
        debug!("Checking if element exists: {}", selector);
        let tab = get_current_tab().map_err(|e| {
            debug!("Failed to get current tab: {}", e);
            return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
        })?;
        let exists = tab.find_element(selector).is_ok();
        info!("Element '{}' exists: {}", selector, exists);
        return Ok(format!("Element exists: {}", exists));
    }
}
