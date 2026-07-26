//! Window list driver
//!
//! This driver provides functionality to list all open windows.
use super::common::list_windows;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for listing windows
#[derive(Debug)]
pub struct WindowControlListDriver;
#[async_trait::async_trait]
impl Driver for WindowControlListDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_list"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "List all open windows"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to see what windows are currently open"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_list"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Found 5 windows:\n1. 微信 (WeChat.exe, PID: 12345)\n2. Visual Studio Code (Code.exe, PID: 23456)\n3. Chrome (chrome.exe, PID: 34567)".to_string();
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
        debug!("Executing window_control_list driver");
        let windows = list_windows()?;
        if windows.is_empty() {
            info!("No windows found");
            return Ok("No windows found".to_string());
        }
        info!("Found {} windows", windows.len());
        let mut result = format!("Found {} windows:\n", windows.len());
        for (i, window) in windows.iter().enumerate() {
            result.push_str(&format!(
                "{}. {} ({} [{}], PID: {})\n",
                i + 1,
                window.title,
                window.process_name,
                if window.is_minimized { "minimized" } else { "visible" },
                window.pid
            ));
        }
        return Ok(result);
    }
}
