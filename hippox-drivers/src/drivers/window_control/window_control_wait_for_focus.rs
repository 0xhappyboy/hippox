//! Window wait for focus driver
//!
//! This driver provides functionality to wait for a window to gain focus.
use super::common::get_focus_window;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info};
/// Driver for waiting for a window to gain focus
#[derive(Debug)]
pub struct WindowControlWaitForFocusDriver;
#[async_trait::async_trait]
impl Driver for WindowControlWaitForFocusDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_wait_for_focus"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Wait for a window to gain focus"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to wait until a specific window becomes active"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Window title to wait for".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("微信".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "process".to_string(),
                param_type: "string".to_string(),
                description: "Process name to wait for".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("WeChat.exe".to_string())),
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
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_wait_for_focus",
            "parameters": {
                "title": "微信",
                "timeout_ms": 10000
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Window gained focus after 1234ms".to_string();
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
        debug!("Executing window_control_wait_for_focus driver");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        let timeout_ms = parameters.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(30000);
        info!("Waiting for focus: title={:?}, process={:?}, timeout={}ms", title, process, timeout_ms);
        let start = Instant::now();
        loop {
            if start.elapsed() > Duration::from_millis(timeout_ms) {
                info!("Timeout waiting for window to gain focus");
                return Err(DriverError::execution("Timeout waiting for window to gain focus"));
            }
            let focused_id = get_focus_window()?;
            use super::common::list_windows;
            let windows = list_windows()?;
            if let Some(focused) = windows.iter().find(|w| w.id == focused_id) {
                let match_title = title.map_or(false, |t| focused.title.to_lowercase().contains(&t.to_lowercase()));
                let match_process = process.map_or(false, |p| focused.process_name.to_lowercase().contains(&p.to_lowercase()));
                if match_title || match_process {
                    let elapsed = start.elapsed().as_millis();
                    info!("Window gained focus after {}ms: {} ({})", elapsed, focused.title, focused.process_name);
                    return Ok(format!("Window gained focus after {}ms", elapsed));
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
