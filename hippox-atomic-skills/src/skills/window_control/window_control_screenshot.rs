//! Window screenshot skill

use super::common::{find_window, get_window_rect};
use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use image::GenericImageView;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct WindowControlScreenshotSkill;

#[async_trait::async_trait]
impl Skill for WindowControlScreenshotSkill {
    fn name(&self) -> &str {
        "window_control_screenshot"
    }

    fn description(&self) -> &str {
        "Take a screenshot of a specified window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to capture an image of a window"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Window title (partial match)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("微信".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "process".to_string(),
                param_type: "string".to_string(),
                description: "Process name".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("WeChat.exe".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "File path to save screenshot".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("./screenshot.png".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_screenshot",
            "parameters": {
                "title": "微信",
                "path": "./wechat.png"
            }
        })
    }

    fn example_output(&self) -> String {
        "Screenshot saved to ./wechat.png".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Window
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
        let window_id = find_window(title, process)?;
        let rect = get_window_rect(window_id)?;
        #[cfg(target_os = "windows")]
        {
            use xcap::Monitor;
            let monitors =
                Monitor::all().map_err(|e| anyhow::anyhow!("Failed to get monitors: {}", e))?;
            let monitor = monitors
                .iter()
                .find(|m| m.is_primary().map(|is_primary| is_primary).unwrap_or(false))
                .ok_or_else(|| anyhow::anyhow!("No primary monitor found"))?;
            let full_image = monitor
                .capture_image()
                .map_err(|e| anyhow::anyhow!("Failed to capture screenshot: {}", e))?;
            let cropped = full_image.view(rect.x as u32, rect.y as u32, rect.width, rect.height);
            let cropped_image = cropped.to_image();
            cropped_image.save(path)?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (rect, path);
            anyhow::bail!("Screenshot not implemented on this platform yet");
        }
        Ok(format!("Screenshot saved to {}", path))
    }
}
