//! OCR (Optical Character Recognition) driver module
//!
//! This module provides functionality to extract text from images using Tesseract OCR.
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, file_exists,
    types::{Driver, DriverParameter},
};
/// Driver for OCR text extraction from images
#[derive(Debug)]
pub struct OcrDriver;
#[async_trait::async_trait]
impl Driver for OcrDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "ocr";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Extract text from images using OCR (requires Tesseract)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to extract text from images, scanned documents, or PDFs. \
        Requires Tesseract OCR to be installed on the system.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the image or PDF file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/document.png".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "language".to_string(),
                param_type: "string".to_string(),
                description: "Language code (eng, chi_sim, etc.)".to_string(),
                required: false,
                default: Some(Value::String("eng".to_string())),
                example: Some(Value::String("chi_sim".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "ocr",
            "parameters": {
                "path": "/documents/scan.jpg",
                "language": "eng"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Extracted text: Hello World! This is OCR text.".to_string();
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
        debug!("Executing ocr driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting OCR text extraction".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let language = parameters.get("language").and_then(|v| v.as_str()).unwrap_or("eng");
        debug!("Running OCR on: {}, language: {}", path, language);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Image: {}, language: {}", path, language)));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        // Validate file exists
        if !file_exists(path) {
            return Err(DriverError::execution(format!("File not found: {}", path)));
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("File verified: {}", path)));
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Running Tesseract OCR...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        // Run Tesseract OCR
        #[cfg(not(target_os = "windows"))]
        {
            let output = std::process::Command::new("tesseract")
                .args([path, "stdout", "-l", language])
                .output()
                .map_err(|e| DriverError::execution(format!("Tesseract not found: {}. Please install Tesseract OCR.", e)))?;
            if !output.status.success() {
                return Err(DriverError::execution(format!("OCR failed: {}", String::from_utf8_lossy(&output.stderr))));
            }
            let text = String::from_utf8(output.stdout).map_err(|e| DriverError::execution(format!("Invalid UTF-8 output: {}", e)))?;
            if text.trim().is_empty() {
                return Err(DriverError::execution("No text found in image".to_string()));
            }
            let result_msg = format!("Extracted text: {}", text.trim());
            if let Some(cb) = callback {
                cb.on_log(task_id.clone(), driver_index, Some(format!("Result: {}", result_msg)));
                cb.on_progress(task_id.clone(), driver_index, Some(100), None);
                cb.on_complete(task_id.clone(), driver_index, Some("ocr".to_string()), Some(result_msg.clone()));
            }
            info!("OCR completed for: {}", path);
            return Ok(result_msg);
        }
        #[cfg(target_os = "windows")]
        {
            let output = std::process::Command::new("tesseract")
                .args([path, "stdout", "-l", language])
                .output()
                .map_err(|e| DriverError::execution(format!("Tesseract not found: {}", e)))?;
            if !output.status.success() {
                return Err(DriverError::execution(format!("OCR failed: {}", String::from_utf8_lossy(&output.stderr))));
            }
            let text = String::from_utf8_lossy(&output.stdout);
            if text.trim().is_empty() {
                return Err(DriverError::execution("No text found in image".to_string()));
            }
            let result_msg = format!("Extracted text: {}", text.trim());
            if let Some(cb) = callback {
                cb.on_log(task_id.clone(), driver_index, Some(format!("Result: {}", result_msg)));
                cb.on_progress(task_id.clone(), driver_index, Some(100), None);
                cb.on_complete(task_id.clone(), driver_index, Some("ocr".to_string()), Some(result_msg.clone()));
            }
            info!("OCR completed for: {}", path);
            return Ok(result_msg);
        }
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
