//! QR Code parsing driver module
//!
//! This module provides functionality to parse/read content from QR code images.
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, file_exists,
    types::{Driver, DriverParameter},
};
/// Driver for parsing QR codes from images
#[derive(Debug)]
pub struct QrCodeParseDriver;
#[async_trait::async_trait]
impl Driver for QrCodeParseDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "qrcode_parse";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Parse/read content from a QR code image";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to decode QR codes from images.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the QR code image".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/path/to/qrcode.png".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "qrcode_parse",
            "parameters": {
                "path": "/images/qrcode.png"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "QR Code content: https://example.com".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Media;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing qrcode_parse driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting QR code parsing".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        debug!("Parsing QR code from: {}", path);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("QR code path: {}", path)));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        // Validate file exists
        if !file_exists(path) {
            return Err(DriverError::execution(format!("QR code image not found: {}", path)));
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("File verified: {}", path)));
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Opening and decoding QR code...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        // Open and decode QR code
        use rqrr;
        let img = image::open(path).map_err(|e| DriverError::execution(format!("Failed to open image: {}", e)))?;
        let img = img.to_luma8();
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Detecting QR code grids...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(65), None);
        }
        let mut prepared = rqrr::PreparedImage::prepare(img);
        let grids = prepared.detect_grids();
        if grids.is_empty() {
            return Err(DriverError::execution("No QR code found in image".to_string()));
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Decoding QR code content...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(80), None);
        }
        let decoded = grids[0].decode().map_err(|e| DriverError::execution(format!("Failed to decode QR code: {}", e)))?;
        let result_msg = format!("QR Code content: {:?}", decoded);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Result: {}", result_msg)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("qrcode_parse".to_string()), Some(result_msg.clone()));
        }
        info!("QR code parsing completed: {}", path);
        return Ok(result_msg);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
