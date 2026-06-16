//! Window OCR region skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{find_window, get_window_rect};
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WindowControlOcrRegionSkill;

#[async_trait::async_trait]
impl Skill for WindowControlOcrRegionSkill {
    fn name(&self) -> &str {
        "window_control_ocr_region"
    }

    fn description(&self) -> &str {
        "Recognize text in a specified region of a window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to extract text from a specific area of a window"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Window title (partial match)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("记事本".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "process".to_string(),
                param_type: "string".to_string(),
                description: "Process name".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("notepad.exe".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "X offset from window left".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Y offset from window top".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "width".to_string(),
                param_type: "integer".to_string(),
                description: "Width of region".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(200.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "height".to_string(),
                param_type: "integer".to_string(),
                description: "Height of region".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_ocr_region",
            "parameters": {
                "title": "记事本",
                "x": 10,
                "y": 50,
                "width": 200,
                "height": 30
            }
        })
    }

    fn example_output(&self) -> String {
        "Recognized text: Hello World".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Window
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        let x = parameters.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let y = parameters.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let width = parameters.get("width").and_then(|v| v.as_u64());
        let height = parameters.get("height").and_then(|v| v.as_u64());
        let window_id = find_window(title, process)?;
        let rect = get_window_rect(window_id)?;
        let screen_x = rect.x + x;
        let screen_y = rect.y + y;
        let capture_width = width.unwrap_or(rect.width as u64 - x as u64);
        let capture_height = height.unwrap_or(rect.height as u64 - y as u64);
        let _ = (screen_x, screen_y, capture_width, capture_height);
        Ok(format!("OCR region captured (implementation pending)"))
    }
}
