//! Image batch convert skill

use super::common::get_format_from_extension;
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct ImageBatchConvertDriver;

#[async_trait::async_trait]
impl Driver for ImageBatchConvertDriver {
    fn name(&self) -> &str {
        "image_batch_convert"
    }

    fn description(&self) -> &str {
        "Batch convert all images in a directory to a target format"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to convert multiple images at once. \
        Specify input directory, output directory, and target format."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "input_dir".to_string(),
                param_type: "string".to_string(),
                description: "Input directory containing images".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/photos/input".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "output_dir".to_string(),
                param_type: "string".to_string(),
                description: "Output directory for converted images".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/photos/output".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "format".to_string(),
                param_type: "string".to_string(),
                description: "Target format: jpg, png, webp, bmp, gif".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("webp".to_string())),
                enum_values: Some(vec![
                    "jpg".to_string(),
                    "jpeg".to_string(),
                    "png".to_string(),
                    "webp".to_string(),
                    "bmp".to_string(),
                    "gif".to_string(),
                ]),
            },
            DriverParameter {
                name: "quality".to_string(),
                param_type: "integer".to_string(),
                description: "Quality for lossy formats (1-100)".to_string(),
                required: false,
                default: Some(Value::Number(85.into())),
                example: Some(Value::Number(80.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "image_batch_convert",
            "parameters": {
                "input_dir": "/photos/raw",
                "output_dir": "/photos/webp",
                "format": "webp",
                "quality": 85
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully converted 15 images to webp".to_string()
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
                Some("Starting batch image conversion".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }

        let input_dir = parameters
            .get("input_dir")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'input_dir' parameter"))?;
        let output_dir = parameters
            .get("output_dir")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'output_dir' parameter"))?;
        let target_format = parameters
            .get("format")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'format' parameter"))?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!(
                    "Input: {}, output: {}, format: {}",
                    input_dir, output_dir, target_format
                )),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }

        let input_path = Path::new(input_dir);
        let output_path = Path::new(output_dir);

        if !input_path.exists() {
            anyhow::bail!("Input directory not found: {}", input_dir);
        }

        if !output_path.exists() {
            fs::create_dir_all(output_path)?;
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some(format!("Created output directory: {}", output_dir)),
                );
            }
        }

        let format = get_format_from_extension(&format!("dummy.{}", target_format))
            .ok_or_else(|| anyhow::anyhow!("Unsupported format: {}", target_format))?;

        let image_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "avif"];
        let mut converted = 0;
        let mut total = 0;

        if let Some(cb) = cb {
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }

        for entry in fs::read_dir(input_path)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase());

            if let Some(ext) = ext {
                if !image_extensions.contains(&ext.as_str()) {
                    continue;
                }
            }

            total += 1;
        }
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Found {} images to convert", total)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        let mut processed = 0;
        for entry in fs::read_dir(input_path)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase());
            if let Some(ext) = ext {
                if !image_extensions.contains(&ext.as_str()) {
                    continue;
                }
            }
            let file_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;
            let output_file = output_path.join(format!("{}.{}", file_name, target_format));
            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some(format!(
                        "Converting {}/{}: {}",
                        processed + 1,
                        total,
                        path.display()
                    )),
                );
                let progress = 40 + ((processed as f32 / total as f32) * 50.0) as u32;
                cb.on_progress(task_id.clone(), driver_index, Some(progress.min(95)), None);
            }
            match convert_image(&path, &output_file, format) {
                Ok(_) => {
                    converted += 1;
                    processed += 1;
                }
                Err(e) => {
                    if let Some(cb) = cb {
                        cb.on_log(
                            task_id.clone(),
                            driver_index,
                            Some(format!("Failed to convert {}: {}", path.display(), e)),
                        );
                    }
                    processed += 1;
                }
            }
        }
        let result_msg = format!(
            "Successfully converted {} images to {}",
            converted, target_format
        );
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("Result: {}", result_msg)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                driver_index,
                Some("image_batch_convert".to_string()),
                Some(result_msg.clone()),
            );
        }
        Ok(result_msg)
    }
}

fn convert_image(input: &Path, output: &Path, format: image::ImageFormat) -> Result<()> {
    let img =
        image::open(input).map_err(|e| anyhow::anyhow!("Failed to open {:?}: {}", input, e))?;

    img.save_with_format(output, format)
        .map_err(|e| anyhow::anyhow!("Failed to save {:?}: {}", output, e))?;

    Ok(())
}
