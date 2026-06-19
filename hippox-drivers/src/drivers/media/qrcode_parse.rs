//! QR Code parse skill

use crate::{
    DriverCallback, DriverCategory, DriverContext, file_exists,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct QrCodeParseDriver;

#[async_trait::async_trait]
impl Driver for QrCodeParseDriver {
    fn name(&self) -> &str {
        "qrcode_parse"
    }

    fn description(&self) -> &str {
        "Parse/read content from a QR code image"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to decode QR codes from images."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the QR code image".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/path/to/qrcode.png".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "qrcode_parse",
            "parameters": {
                "path": "/images/qrcode.png"
            }
        })
    }

    fn example_output(&self) -> String {
        "QR Code content: https://example.com".to_string()
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
                Some("Starting QR code parsing".to_string()),
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
                Some(format!("QR code path: {}", path)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }

        if !file_exists(path) {
            anyhow::bail!("QR code image not found: {}", path);
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!("File verified: {}", path)),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Opening and decoding QR code...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }

        let img = image::open(path).map_err(|e| anyhow::anyhow!("Failed to open image: {}", e))?;

        let img = img.to_luma8();

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Detecting QR code grids...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(65), None);
        }

        let mut prepared = rqrr::PreparedImage::prepare(img);
        let grids = prepared.detect_grids();

        if grids.is_empty() {
            anyhow::bail!("No QR code found in image");
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Decoding QR code content...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(80), None);
        }

        let decoded = grids[0]
            .decode()
            .map_err(|e| anyhow::anyhow!("Failed to decode QR code: {}", e))?;

        let result_msg = format!("QR Code content: {:?}", decoded);

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
                Some("qrcode_parse".to_string()),
                Some(result_msg.clone()),
            );
        }

        Ok(result_msg)
    }
}
