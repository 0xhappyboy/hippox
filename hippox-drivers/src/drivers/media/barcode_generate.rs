//! Barcode generation driver module
//!
//! This module provides functionality to generate barcodes (Code128, EAN-13, etc.)
//! and save them as PNG images.
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
/// Driver for generating barcodes
#[derive(Debug)]
pub struct BarcodeGenerateDriver;
#[async_trait::async_trait]
impl Driver for BarcodeGenerateDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "barcode_generate";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Generate a barcode (Code128, EAN-13, etc.)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to create barcodes for product codes, ISBN, etc.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "content".to_string(),
                param_type: "string".to_string(),
                description: "Content to encode in the barcode".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("123456789012".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "destination".to_string(),
                param_type: "string".to_string(),
                description: "Output file path (PNG format)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/barcode.png".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "barcode_type".to_string(),
                param_type: "string".to_string(),
                description: "Barcode type: code128, ean13, upca, code39".to_string(),
                required: false,
                default: Some(Value::String("code128".to_string())),
                example: Some(Value::String("ean13".to_string())),
                enum_values: Some(vec!["code128".to_string(), "ean13".to_string(), "upca".to_string(), "code39".to_string()]),
            },
            DriverParameter {
                name: "height".to_string(),
                param_type: "integer".to_string(),
                description: "Height of barcode in pixels".to_string(),
                required: false,
                default: Some(Value::Number(80.into())),
                example: Some(Value::Number(120.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "barcode_generate",
            "parameters": {
                "content": "9781234567897",
                "destination": "/output/barcode.png",
                "barcode_type": "ean13"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Barcode generated successfully at /output/barcode.png".to_string();
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
        debug!("Executing barcode_generate driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting barcode generation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let content = parameters.get("content").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("content"))?;
        let destination = parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        let barcode_type = parameters.get("barcode_type").and_then(|v| v.as_str()).unwrap_or("code128");
        let height = parameters.get("height").and_then(|v| v.as_u64()).unwrap_or(80) as u32;
        debug!("Generating barcode: content='{}', type='{}', height={}", content, barcode_type, height);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Content: {}, type: {}, height: {}", content, barcode_type, height)));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Encoding barcode...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        // Encode barcode based on type
        let encoded = match barcode_type {
            "code128" => {
                use barcoders::sym::code128::Code128;
                let encoder = Code128::new(content).map_err(|e| DriverError::execution(format!("Failed to create Code128: {}", e)))?;
                encoder.encode()
            }
            "ean13" => {
                use barcoders::sym::ean13::EAN13;
                let encoder = EAN13::new(content).map_err(|e| DriverError::execution(format!("Failed to create EAN13: {}", e)))?;
                encoder.encode()
            }
            "code39" => {
                use barcoders::sym::code39::Code39;
                let encoder = Code39::new(content).map_err(|e| DriverError::execution(format!("Failed to create Code39: {}", e)))?;
                encoder.encode()
            }
            _ => {
                return Err(DriverError::execution(format!("Unsupported barcode type: {}", barcode_type)));
            }
        };
        let width = encoded.len() as u32;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Encoded length: {} modules", width)));
            cb.on_progress(task_id.clone(), driver_index, Some(60), None);
        }
        // Render barcode image
        let module_width = 2;
        let image_width = width * module_width;
        use image::{ImageBuffer, Rgb};
        let mut image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(image_width, height);
        for pixel in image.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Rendering barcode image...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(75), None);
        }
        for (i, &bit) in encoded.iter().enumerate() {
            if bit == 1 {
                let i_usize = i as u32;
                for y in 0..height {
                    for x in 0..module_width {
                        image.put_pixel((i_usize * module_width) + x, y, Rgb([0, 0, 0]));
                    }
                }
            }
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Saving barcode...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(90), None);
        }
        image.save(destination).map_err(|e| DriverError::execution(format!("Failed to save barcode: {}", e)))?;
        let result_msg = format!("Barcode generated successfully at {}", destination);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Result: {}", result_msg)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("barcode_generate".to_string()), Some(result_msg.clone()));
        }
        info!("Barcode generated successfully: {}", destination);
        return Ok(result_msg);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("content").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("content"))?;
        parameters.get("destination").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("destination"))?;
        return Ok(());
    }
}
