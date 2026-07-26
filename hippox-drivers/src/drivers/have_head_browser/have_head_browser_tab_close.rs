//! Browser close tab skill
//!
//! This driver provides functionality to close the current browser tab.
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
/// Driver for closing the current tab
#[derive(Debug)]
pub struct HaveHeadBrowserTabCloseDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserTabCloseDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_tab_close"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Close the current browser tab"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to close the current tab"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_tab_close"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Closed current tab".to_string();
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
        debug!("Executing have_head_browser_tab_close driver");
        let tab = get_current_tab().map_err(|e| {
            debug!("Failed to get current tab: {}", e);
            return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
        })?;
        tab.close(false).map_err(|e| {
            warn!("Failed to close tab: {}", e);
            return crate::DriverError::execution(format!("Failed to close tab: {}", e));
        })?;
        clear_current_tab();
        info!("Closed current tab");
        return Ok("Closed current tab".to_string());
    }
}
