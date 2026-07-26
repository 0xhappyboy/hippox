//! Browser switch tab skill
//!
//! This driver provides functionality to switch between browser tabs by index.
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
/// Driver for switching between tabs
#[derive(Debug)]
pub struct HaveHeadBrowserTabSwitchDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserTabSwitchDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_tab_switch"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Switch to a different browser tab by index"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to switch between open tabs. Index 0 is the first tab."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "index".to_string(),
            param_type: "integer".to_string(),
            description: "Tab index to switch to (0-based)".to_string(),
            required: true,
            default: None,
            example: Some(Value::Number(0.into())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_tab_switch",
            "parameters": {
                "index": 0
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Switched to tab 0".to_string();
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
        debug!("Executing have_head_browser_tab_switch driver");
        let index = parameters.get("index").and_then(|v| v.as_u64()).ok_or_else(|| {
            debug!("Missing required parameter: index (0-based)");
            return crate::DriverError::missing_parameter("index");
        })? as usize;
        debug!("Switching to tab index: {}", index);
        let browser = get_or_create_browser().map_err(|e| {
            debug!("Failed to get or create browser: {}", e);
            return crate::DriverError::execution(format!("Failed to get or create browser: {}", e));
        })?;
        let tabs_guard = browser.get_tabs();
        let tabs = tabs_guard.lock().unwrap();
        if index >= tabs.len() {
            warn!("Tab index {} out of range ({} tabs available)", index, tabs.len());
            return Err(crate::DriverError::validation("index", format!("Tab index {} out of range ({} tabs available)", index, tabs.len())));
        }
        let target_tab = tabs[index].clone();
        set_current_tab(target_tab);
        info!("Switched to tab {}", index);
        return Ok(format!("Switched to tab {}", index));
    }
}
