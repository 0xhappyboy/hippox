//! Browser screenshot skill - capture visible browser window
//!
//! This driver provides functionality to take screenshots of the current page
//! and save them to files in PNG format.
use super::shared::*;
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use tracing::{debug, info, warn};
/// Driver for taking screenshots
#[derive(Debug)]
pub struct HaveHeadBrowserScreenshotDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserScreenshotDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_screenshot"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Take a screenshot of the current page and save to file"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to capture visual state of the current page"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "File path to save screenshot (PNG format)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("screenshot.png".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "full_page".to_string(),
                param_type: "boolean".to_string(),
                description: "Capture full page (not just viewport)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_screenshot",
            "parameters": {
                "path": "./screenshot.png"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Screenshot saved to ./screenshot.png".to_string();
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
        debug!("Executing have_head_browser_screenshot driver");
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing required parameter: path");
            return crate::DriverError::missing_parameter("path");
        })?;
        let full_page = parameters.get("full_page").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Taking screenshot: {} (full_page: {})", path, full_page);
        let tab = get_current_tab().map_err(|e| {
            debug!("Failed to get current tab: {}", e);
            return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
        })?;
        let png_data = if full_page {
            tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true).map_err(|e| {
                warn!("Failed to capture full page screenshot: {}", e);
                return crate::DriverError::execution(format!("Failed to capture full page screenshot: {}", e));
            })?
        } else {
            tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, false).map_err(|e| {
                warn!("Failed to capture viewport screenshot: {}", e);
                return crate::DriverError::execution(format!("Failed to capture viewport screenshot: {}", e));
            })?
        };
        fs::write(path, &png_data).map_err(|e| {
            warn!("Failed to save screenshot: {}", e);
            return crate::DriverError::io(format!("Failed to save screenshot: {}", e));
        })?;
        info!("Screenshot saved to {} ({} bytes)", path, png_data.len());
        return Ok(format!("Screenshot saved to {}", path));
    }
}
