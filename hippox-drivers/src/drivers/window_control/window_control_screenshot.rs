//! Window screenshot driver
//!
//! This driver provides functionality to take a screenshot of a specified window.
use super::common::{find_window, get_window_rect};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use image::GenericImageView;
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for taking a window screenshot
#[derive(Debug)]
pub struct WindowControlScreenshotDriver;
#[async_trait::async_trait]
impl Driver for WindowControlScreenshotDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_screenshot"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Take a screenshot of a specified window"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to capture an image of a window"
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
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "File path to save screenshot".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("./screenshot.png".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_screenshot",
            "parameters": {
                "title": "微信",
                "path": "./wechat.png"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Screenshot saved to ./wechat.png".to_string();
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
        debug!("Executing window_control_screenshot driver");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        info!("Taking screenshot: title={:?}, process={:?}, path={}", title, process, path);
        let window_id = find_window(title, process)?;
        let rect = get_window_rect(window_id)?;
        info!("Window rect: x={}, y={}, w={}, h={}", rect.x, rect.y, rect.width, rect.height);
        #[cfg(target_os = "windows")]
        {
            use xcap::Monitor;
            let monitors = Monitor::all().map_err(|e| DriverError::execution(format!("Failed to get monitors: {}", e)))?;
            let monitor =
                monitors.iter().find(|m| m.is_primary().unwrap_or(false)).ok_or_else(|| DriverError::execution("No primary monitor found"))?;
            let full_image = monitor.capture_image().map_err(|e| DriverError::execution(format!("Failed to capture screenshot: {}", e)))?;
            let cropped = full_image.view(rect.x as u32, rect.y as u32, rect.width, rect.height);
            let cropped_image = cropped.to_image();
            cropped_image.save(path).map_err(|e| DriverError::execution(format!("Failed to save screenshot: {}", e)))?;
            info!("Screenshot saved to {}", path);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (rect, path);
            info!("Screenshot not implemented on this platform yet");
            return Err(DriverError::execution("Screenshot not implemented on this platform yet"));
        }
        return Ok(format!("Screenshot saved to {}", path));
    }
}
