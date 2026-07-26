//! Window OCR region driver
//!
//! This driver provides functionality to recognize text in a specified region of a window.
use super::common::{find_window, get_window_rect};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for OCR on a window region
#[derive(Debug)]
pub struct WindowControlOcrRegionDriver;
#[async_trait::async_trait]
impl Driver for WindowControlOcrRegionDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "window_control_ocr_region"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Recognize text in a specified region of a window"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to extract text from a specific area of a window"
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
                example: Some(Value::String("记事本".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "process".to_string(),
                param_type: "string".to_string(),
                description: "Process name".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("notepad.exe".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "X offset from window left".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Y offset from window top".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "width".to_string(),
                param_type: "integer".to_string(),
                description: "Width of region".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(200.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "height".to_string(),
                param_type: "integer".to_string(),
                description: "Height of region".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "window_control_ocr_region",
            "parameters": {
                "title": "记事本",
                "x": 10,
                "y": 50,
                "width": 200,
                "height": 30
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Recognized text: Hello World".to_string();
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
        debug!("Executing window_control_ocr_region driver");
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        let x = parameters.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let y = parameters.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let width = parameters.get("width").and_then(|v| v.as_u64());
        let height = parameters.get("height").and_then(|v| v.as_u64());
        info!("OCR region: title={:?}, process={:?}, x={}, y={}, width={:?}, height={:?}", title, process, x, y, width, height);
        let window_id = find_window(title, process)?;
        let rect = get_window_rect(window_id)?;
        let screen_x = rect.x + x;
        let screen_y = rect.y + y;
        let capture_width = width.unwrap_or(rect.width as u64 - x as u64);
        let capture_height = height.unwrap_or(rect.height as u64 - y as u64);
        info!("Screen region: x={}, y={}, w={}, h={}", screen_x, screen_y, capture_width, capture_height);
        // TODO: Implement actual OCR using tesseract or similar
        // For now, return a placeholder
        info!("OCR implementation pending");
        return Ok("OCR region captured (implementation pending)".to_string());
    }
}
