//! Browser get URL skill
//!
//! This driver provides functionality to get the URL of the current page.
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
/// Driver for getting the current page URL
#[derive(Debug)]
pub struct HaveHeadBrowserGetUrlDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserGetUrlDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_get_url"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the current page URL"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to check what page the browser is currently on"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_get_url"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Current URL: https://www.google.com".to_string();
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
        debug!("Executing have_head_browser_get_url driver");
        let tab = get_current_tab().map_err(|e| {
            debug!("Failed to get current tab: {}", e);
            return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
        })?;
        let url = tab.get_url();
        info!("Current URL: {}", url);
        return Ok(format!("Current URL: {}", url));
    }
}
