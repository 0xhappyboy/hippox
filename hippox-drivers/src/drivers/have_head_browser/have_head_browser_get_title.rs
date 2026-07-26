//! Browser get page title skill
//!
//! This driver provides functionality to get the title of the current page.
use super::shared::*;
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter,},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting the current page title
#[derive(Debug)]
pub struct HaveHeadBrowserGetTitleDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserGetTitleDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_get_title"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the current page title"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the title of the current page"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_get_title"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Page title: Google".to_string();
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
        debug!("Executing have_head_browser_get_title driver");
        let tab = get_current_tab().map_err(|e| {
            debug!("Failed to get current tab: {}", e);
            return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
        })?;
        let title = tab.get_title().unwrap_or_else(|_| "Unknown".to_string());
        info!("Page title: {}", title);
        return Ok(format!("Page title: {}", title));
    }
}
