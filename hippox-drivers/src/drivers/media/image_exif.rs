//! Image EXIF metadata skill

use crate::{
    DriverCallback, DriverCategory, DriverContext, file_exists,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug)]
pub struct ImageExifDriver;

#[async_trait::async_trait]
impl Driver for ImageExifDriver {
    fn name(&self) -> &str {
        "image_exif"
    }

    fn description(&self) -> &str {
        "Read EXIF metadata from images (GPS, camera info, date taken, etc.)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to extract EXIF metadata from photos."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the image file".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/path/to/photo.jpg".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_exif",
            "parameters": {
                "path": "/photos/DSC_001.jpg"
            }
        })
    }

    fn example_output(&self) -> String {
        r#"{"make":"Nikon","model":"D850","iso":400}"#.to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Media
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.driver_name())
            .map(String::from);
        let cb = callback;
        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Starting EXIF metadata extraction".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Image path: {}", path)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        if !file_exists(path) {
            anyhow::bail!("Image not found: {}", path);
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("File verified: {}", path)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        let file = File::open(path)?;
        let mut bufreader = BufReader::new(file);
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Parsing EXIF data...".to_string()),
            );
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
                if let Some(cb) = cb {
                    cb.on_log(
                        task_id.clone(),
                        driver_index,
                        Some("No EXIF data found".to_string()),
                    );
                    cb.on_progress(task_id.clone(), driver_index, Some(100), None);
                    cb.on_complete(
                        task_id.clone(),
                        driver_index,
                        Some("image_exif".to_string()),
                        Some(result.clone()),
                    );
                }
                return Ok(result);
            }
        };
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Found {} EXIF fields", exif.fields().count())),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(70), None);
        }
        let mut result = serde_json::Map::new();
        result.insert("path".to_string(), json!(path));
        for field in exif.fields() {
            let tag_name = format!("{:?}", field.tag).to_lowercase();
            let value = field.display_value().with_unit(&exif).to_string();
            result.insert(tag_name, json!(value));
        }
        result.insert("field_count".to_string(), json!(exif.fields().count()));
        let result_str = serde_json::to_string_pretty(&result)?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("EXIF extraction complete".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                driver_index,
                Some("image_exif".to_string()),
                Some(result_str.clone()),
            );
        }
        Ok(result_str)
    }
}
