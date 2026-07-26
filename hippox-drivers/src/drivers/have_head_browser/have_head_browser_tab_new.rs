//! Browser new tab skill
//!
//! This driver provides functionality to open a new browser tab.
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
/// Driver for opening a new tab
#[derive(Debug)]
pub struct HaveHeadBrowserTabNewDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserTabNewDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_tab_new"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Open a new browser tab"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to open a new tab without closing the current one"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "url".to_string(),
            param_type: "string".to_string(),
            description: "URL to open in the new tab (optional)".to_string(),
            required: false,
            default: None,
            example: Some(Value::String("about:blank".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_tab_new",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Opened new tab".to_string();
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
        debug!("Executing have_head_browser_tab_new driver");
        let browser = get_or_create_browser().map_err(|e| {
            debug!("Failed to get or create browser: {}", e);
            return crate::DriverError::execution(format!("Failed to get or create browser: {}", e));
        })?;
        debug!("Creating new tab");
        let new_tab = browser.new_tab().map_err(|e| {
            warn!("Failed to create new tab: {}", e);
            return crate::DriverError::execution(format!("Failed to create new tab: {}", e));
        })?;
        if let Some(url) = parameters.get("url").and_then(|v| v.as_str()) {
            debug!("Navigating new tab to: {}", url);
            new_tab.navigate_to(url).map_err(|e| {
                warn!("Failed to navigate: {}", e);
                return crate::DriverError::execution(format!("Failed to navigate: {}", e));
            })?;
            new_tab.wait_until_navigated().map_err(|e| {
                warn!("Navigation timeout: {}", e);
                return crate::DriverError::execution(format!("Navigation timeout: {}", e));
            })?;
        }
        set_current_tab(new_tab);
        info!("Opened new tab");
        return Ok("Opened new tab".to_string());
    }
}
