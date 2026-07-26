//! Browser back navigation skill
//!
//! This driver provides functionality to navigate back to the previous page
//! in the browser history.
use super::shared::*;
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter, },
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for navigating back in browser history
#[derive(Debug)]
pub struct HaveHeadBrowserBackDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserBackDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_back"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Navigate back to the previous page"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to go back to the previous page in history"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_back"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Navigated back".to_string();
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
        debug!("Executing have_head_browser_back driver");
        let tab = get_current_tab().map_err(|e| {
            debug!("Failed to get current tab: {}", e);
            return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
        })?;
        debug!("Navigating back");
        tab.evaluate("window.history.back()", false).map_err(|e| {
            warn!("Failed to navigate back: {}", e);
            return crate::DriverError::execution(format!("Failed to navigate back: {}", e));
        })?;
        wait_for_stable(&tab, 1000).await;
        info!("Navigated back successfully");
        return Ok("Navigated back".to_string());
    }
}
