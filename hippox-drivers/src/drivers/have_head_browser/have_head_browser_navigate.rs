//! Browser navigation skill - navigate to URL in visible browser window
//!
//! This driver provides functionality to navigate to a URL in the visible
//! browser window.
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
/// Driver for navigating to URLs
#[derive(Debug)]
pub struct HaveHeadBrowserNavigateDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserNavigateDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_navigate"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Navigate to a URL in the visible browser window"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to open a web page in the browser. A visible browser window will pop up."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "url".to_string(),
                param_type: "string".to_string(),
                description: "URL to navigate to (e.g., https://example.com)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("https://www.google.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "wait_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Milliseconds to wait after navigation (default: 2000)".to_string(),
                required: false,
                default: Some(Value::Number(2000.into())),
                example: Some(Value::Number(3000.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_navigate",
            "parameters": {
                "url": "https://www.google.com"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Navigated to https://www.google.com".to_string();
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
        debug!("Executing have_head_browser_navigate driver");
        let url = parameters.get("url").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing required parameter: url");
            return crate::DriverError::missing_parameter("url");
        })?;
        let wait_ms = parameters.get("wait_ms").and_then(|v| v.as_u64()).unwrap_or(2000);
        debug!("Navigating to: {} (wait_ms: {})", url, wait_ms);
        let tab = get_current_tab().map_err(|e| {
            debug!("Failed to get current tab: {}", e);
            return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
        })?;
        tab.navigate_to(url).map_err(|e| {
            warn!("Failed to navigate: {}", e);
            return crate::DriverError::execution(format!("Failed to navigate: {}", e));
        })?;
        tab.wait_until_navigated().map_err(|e| {
            warn!("Navigation timeout: {}", e);
            return crate::DriverError::execution(format!("Navigation timeout: {}", e));
        })?;
        wait_for_stable(&tab, wait_ms).await;
        info!("Navigated to {}", url);
        return Ok(format!("Navigated to {}", url));
    }
}
