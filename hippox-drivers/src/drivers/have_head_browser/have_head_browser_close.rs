//! Browser close skill
//!
//! This driver provides functionality to close the browser window completely.
use super::shared::*;
use crate::{DriverCallback, DriverContext, DriverResult};
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter, },
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for closing the browser
#[derive(Debug)]
pub struct HaveHeadBrowserCloseDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserCloseDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_close"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Close the browser window completely"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to close the browser when no longer needed"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_close"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Browser closed".to_string();
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
        debug!("Executing have_head_browser_close driver");
        close_browser().map_err(|e| {
            debug!("Failed to close browser: {}", e);
            return crate::DriverError::execution(format!("Failed to close browser: {}", e));
        })?;
        info!("Browser closed");
        return Ok("Browser closed".to_string());
    }
}
