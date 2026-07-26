//! Image EXIF metadata driver module
//!
//! This module provides functionality to read EXIF metadata from images
//! including GPS, camera information, and date taken.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult, file_exists,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use tracing::{debug, info};
/// Driver for extracting EXIF metadata from images
#[derive(Debug)]
pub struct ImageExifDriver;
#[async_trait::async_trait]
impl Driver for ImageExifDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "image_exif";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Read EXIF metadata from images (GPS, camera info, date taken, etc.)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill to extract EXIF metadata from photos.";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the image file".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/path/to/photo.jpg".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "image_exif",
            "parameters": {
                "path": "/photos/DSC_001.jpg"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"{"make":"Nikon","model":"D850","iso":400}"#.to_string();
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
        debug!("Executing image_exif driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting EXIF metadata extraction".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        debug!("Extracting EXIF from: {}", path);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Image path: {}", path)));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        // Validate file exists
        if !file_exists(path) {
            return Err(DriverError::execution(format!("Image not found: {}", path)));
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("File verified: {}", path)));
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        // Open and parse EXIF data
        let file = File::open(path).map_err(|e| DriverError::execution(format!("Failed to open file: {}", e)))?;
        let mut bufreader = BufReader::new(file);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Parsing EXIF data...".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        let exifreader = exif::Reader::new();
        let exif = match exifreader.read_from_container(&mut bufreader) {
            Ok(e) => e,
            Err(_) => {
                let result = json!({
                    "path": path,
                    "error": "No EXIF data found or unsupported format"
                })
                .to_string();
                if let Some(cb) = callback {
                    cb.on_log(task_id.clone(), driver_index, Some("No EXIF data found".to_string()));
                    cb.on_progress(task_id.clone(), driver_index, Some(100), None);
                    cb.on_complete(task_id.clone(), driver_index, Some("image_exif".to_string()), Some(result.clone()));
                }
                info!("No EXIF data found for: {}", path);
                return Ok(result);
            }
        };
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Found {} EXIF fields", exif.fields().count())));
            cb.on_progress(task_id.clone(), driver_index, Some(70), None);
        }
        // Build result
        let mut result = serde_json::Map::new();
        result.insert("path".to_string(), json!(path));
        for field in exif.fields() {
            let tag_name = format!("{:?}", field.tag).to_lowercase();
            let value = field.display_value().with_unit(&exif).to_string();
            result.insert(tag_name, json!(value));
        }
        result.insert("field_count".to_string(), json!(exif.fields().count()));
        let result_str =
            serde_json::to_string_pretty(&result).map_err(|e| DriverError::execution(format!("Failed to serialize EXIF data: {}", e)))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("EXIF extraction complete".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("image_exif".to_string()), Some(result_str.clone()));
        }
        info!("EXIF extraction completed for: {}", path);
        return Ok(result_str);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
