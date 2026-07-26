//! Window set always on top driver
//!
//! This driver provides functionality to set a window to stay always on top.
use super::common::find_window;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for setting a window to always on top
#[derive(Debug)]
pub struct WindowControlSetAlwaysOnTopDriver;
#[async_trait::async_trait]
impl Driver for WindowControlSetAlwaysOnTopDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_set_always_on_top"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Set a window to stay always on top"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to make a window stay on top of all other windows"
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
                example: Some(Value::String("播放器".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "process".to_string(),
                param_type: "string".to_string(),
                description: "Process name".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("mpv.exe".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "enabled".to_string(),
                param_type: "boolean".to_string(),
                description: "Enable always on top (default: true)".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_set_always_on_top",
            "parameters": {
                "title": "播放器",
                "enabled": true
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Window set to always on top".to_string();
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
        debug!("Executing window_control_set_always_on_top driver");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        let enabled = parameters.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
        info!("Setting always on top: title={:?}, process={:?}, enabled={}", title, process, enabled);
        let window_id = find_window(title, process)?;
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::WindowsAndMessaging::{HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos};
            unsafe {
                let hwnd = super::common::u64_to_hwnd(window_id);
                let flags = SWP_NOMOVE | SWP_NOSIZE;
                if enabled {
                    let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, flags);
                } else {
                    let _ = SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, flags);
                }
            }
            info!("Window always on top set to {}: ID={}", enabled, window_id);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = window_id;
            info!("Set always on top not implemented on this platform");
            return Err(DriverError::execution("Set always on top not implemented on this platform"));
        }
        return Ok(format!("Window set to always on top: {}", enabled));
    }
}
