//! Window get focus driver
//!
//! This driver provides functionality to get the currently focused window.
use super::common::get_focus_window;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for getting the focused window
#[derive(Debug)]
pub struct WindowControlGetFocusDriver;
#[async_trait::async_trait]
impl Driver for WindowControlGetFocusDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_get_focus"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Get the currently focused window"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to find out which window is currently active"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_get_focus"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Focused window: 微信 (WeChat.exe)".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Window;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing window_control_get_focus driver");
        let window_id = get_focus_window()?;
        use super::common::list_windows;
        let windows = list_windows()?;
        let window = windows.iter().find(|w| w.id == window_id).ok_or_else(|| DriverError::execution("Window not found"))?;
        info!("Focused window: {} ({})", window.title, window.process_name);
        return Ok(format!("Focused window: {} ({})", window.title, window.process_name));
    }
}
