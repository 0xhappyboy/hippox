//! Browser refresh/reload skill
//!
//! This driver provides functionality to refresh the current page.
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
/// Driver for refreshing the current page
#[derive(Debug)]
pub struct HaveHeadBrowserRefreshDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserRefreshDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_refresh"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Refresh the current page"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to reload the current page"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_refresh"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Page refreshed".to_string();
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
        debug!("Executing have_head_browser_refresh driver");
        let tab = get_current_tab().map_err(|e| {
            debug!("Failed to get current tab: {}", e);
            return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
        })?;
        debug!("Refreshing page");
        tab.reload(false, None).map_err(|e| {
            warn!("Failed to refresh: {}", e);
            return crate::DriverError::execution(format!("Failed to refresh: {}", e));
        })?;
        wait_for_stable(&tab, 2000).await;
        info!("Page refreshed");
        return Ok("Page refreshed".to_string());
    }
}
