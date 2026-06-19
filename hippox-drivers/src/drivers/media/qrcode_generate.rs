//! QR Code generate skill

use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use qrcode::QrCode;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct QrCodeGenerateDriver;

#[async_trait::async_trait]
impl Driver for QrCodeGenerateDriver {
    fn name(&self) -> &str {
        "qrcode_generate"
    }

    fn description(&self) -> &str {
        "Generate a QR code from text or URL"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to create QR codes for URLs, text, or contact information."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "qrcode_generate",
            "parameters": {
                "content": "https://github.com",
                "destination": "/output/qrcode.png",
                "size": 400
            }
        })
    }

    fn example_output(&self) -> String {
        "QR Code generated successfully at /output/qrcode.png".to_string()
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
                Some("Starting QR code generation".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }

        let content = parameters
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;
        let destination = parameters
            .get("destination")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'destination' parameter"))?;
        let size = parameters
            .get("size")
            .and_then(|v| v.as_u64())
            .unwrap_or(300) as u32;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some(format!(
                    "Content: {}, destination: {}, size: {}",
                    content, destination, size
                )),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Generating QR code...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }

        let code = QrCode::new(content)
            .map_err(|e| anyhow::anyhow!("Failed to generate QR code: {}", e))?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Rendering QR code image...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(70), None);
        }

        let image = code
            .render::<image::Luma<u8>>()
            .min_dimensions(size, size)
            .build();

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                driver_index,
                Some("Saving QR code...".to_string()),
            );
            cb.on_progress(task_id.clone(), driver_index, Some(85), None);
        }

        image
            .save(destination)
            .map_err(|e| anyhow::anyhow!("Failed to save QR code: {}", e))?;

        let result_msg = format!("QR Code generated successfully at {}", destination);

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
                Some("qrcode_generate".to_string()),
                Some(result_msg.clone()),
            );
        }

        Ok(result_msg)
    }
}
