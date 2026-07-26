//! QR Code generation driver module
//!
//! This module provides functionality to generate QR codes from text or URLs.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for generating QR codes
#[derive(Debug)]
pub struct QrCodeGenerateDriver;
#[async_trait::async_trait]
impl Driver for QrCodeGenerateDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "qrcode_generate";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Generate a QR code from text or URL";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to create QR codes for URLs, text, or contact information.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "content".to_string(),
                param_type: "string".to_string(),
                description: "Content to encode in the QR code".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("https://example.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Output file path (PNG format)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/qrcode.png".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "size".to_string(),
                param_type: "integer".to_string(),
                description: "Size of the QR code in pixels".to_string(),
                required: false,
                default: Some(Value::Number(300.into())),
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "qrcode_generate",
            "parameters": {
                "content": "https://github.com",
                "destination": "/output/qrcode.png",
                "size": 400
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "QR Code generated successfully at /output/qrcode.png".to_string();
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
        debug!("Executing qrcode_generate driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting QR code generation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let content = parameters.get("content").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("content"))?;
        let destination = parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        let size = parameters.get("size").and_then(|v| v.as_u64()).unwrap_or(300) as u32;
        debug!("Generating QR code: content='{}', size={}", content, size);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Content: {}, destination: {}, size: {}", content, destination, size)));
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Generating QR code...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        // Generate QR code
        use qrcode::QrCode;
        let code = QrCode::new(content).map_err(|e| DriverError::execution(format!("Failed to generate QR code: {}", e)))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Rendering QR code image...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(70), None);
        }
        // Render and save
        let image = code.render::<image::Luma<u8>>().min_dimensions(size, size).build();
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Saving QR code...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(85), None);
        }
        image.save(destination).map_err(|e| DriverError::execution(format!("Failed to save QR code: {}", e)))?;
        let result_msg = format!("QR Code generated successfully at {}", destination);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Result: {}", result_msg)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("qrcode_generate".to_string()), Some(result_msg.clone()));
        }
        info!("QR code generated successfully: {}", destination);
        return Ok(result_msg);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("content").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("content"))?;
        parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        return Ok(());
    }
}
