//! Window send to back driver
//!
//! This driver provides functionality to send a window to the back (behind other windows).
use super::common::find_window;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for sending a window to the back
#[derive(Debug)]
pub struct WindowControlSendToBackDriver;
#[async_trait::async_trait]
impl Driver for WindowControlSendToBackDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_send_to_back"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Send a window to the back (behind other windows)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to send a window behind all other windows"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Window title (partial match)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("微信".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "process".to_string(),
                param_type: "string".to_string(),
                description: "Process name".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("WeChat.exe".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_send_to_back",
            "parameters": {
                "title": "微信"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Window sent to back".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Window;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing window_control_send_to_back driver");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        info!("Sending window to back: title={:?}, process={:?}", title, process);
        let window_id = find_window(title, process)?;
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::WindowsAndMessaging::{HWND_BOTTOM, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos};
            unsafe {
                let hwnd = super::common::u64_to_hwnd(window_id);
                let _ = SetWindowPos(hwnd, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
            }
            info!("Window sent to back: ID={}", window_id);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = window_id;
            info!("Send to back not implemented on this platform");
            return Err(DriverError::execution("Send to back not implemented on this platform"));
        }
        return Ok("Window sent to back".to_string());
    }
}
