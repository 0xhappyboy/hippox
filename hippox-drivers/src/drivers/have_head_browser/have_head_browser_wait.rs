//! Browser wait driver - wait for time or element
//!
//! This driver provides functionality to wait for a specified time or
//! for an element to appear on the current page.
use super::shared::*;
use crate::DriverCallback;
use crate::DriverCategory;
use crate::DriverContext;
use crate::DriverResult;
use crate::types::{Driver, DriverParameter};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
/// Driver for waiting
#[derive(Debug)]
pub struct HaveHeadBrowserWaitDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserWaitDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_wait"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Wait for a specified time or for an element to appear"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to wait for page content to load or animations to complete"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "CSS selector to wait for (optional, if not provided, waits fixed time)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String(".loading-done".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum wait time in milliseconds (default: 30000)".to_string(),
                required: false,
                default: Some(Value::Number(30000.into())),
                example: Some(Value::Number(5000.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "wait_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Fixed wait time in milliseconds (used if selector not provided)".to_string(),
                required: false,
                default: Some(Value::Number(1000.into())),
                example: Some(Value::Number(2000.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_wait",
            "parameters": {
                "wait_ms": 2000
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Waited for 2000ms".to_string();
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
        debug!("Executing have_head_browser_wait driver");
        let selector = parameters.get("selector").and_then(|v| v.as_str());
        let timeout_ms = parameters.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(30000);
        if let Some(sel) = selector {
            debug!("Waiting for element: {} (timeout: {}ms)", sel, timeout_ms);
            let tab = get_current_tab().map_err(|e| {
                debug!("Failed to get current tab: {}", e);
                return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
            })?;
            let start = Instant::now();
            let timeout_dur = Duration::from_millis(timeout_ms);
            loop {
                if start.elapsed() > timeout_dur {
                    warn!("Timeout waiting for element: {}", sel);
                    return Err(crate::DriverError::timeout(Some(format!("{}ms", timeout_ms))));
                }
                match tab.find_element(sel) {
                    Ok(_) => {
                        let elapsed = start.elapsed().as_millis();
                        info!("Element '{}' appeared after {}ms", sel, elapsed);
                        return Ok(format!("Element '{}' appeared after {}ms", sel, elapsed));
                    }
                    Err(_) => {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
            }
        } else {
            let wait_ms = parameters.get("wait_ms").and_then(|v| v.as_u64()).unwrap_or(1000);
            debug!("Waiting for {}ms", wait_ms);
            tokio::time::sleep(Duration::from_millis(wait_ms)).await;
            info!("Waited for {}ms", wait_ms);
            return Ok(format!("Waited for {}ms", wait_ms));
        }
    }
}
