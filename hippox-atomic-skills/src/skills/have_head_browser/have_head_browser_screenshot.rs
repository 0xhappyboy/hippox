//! Browser screenshot skill - capture visible browser window

use anyhow::Result;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::*;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct HaveHeadBrowserScreenshotSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserScreenshotSkill {
    fn name(&self) -> &str {
        "have_head_browser_screenshot"
    }

    fn description(&self) -> &str {
        "Take a screenshot of the current page and save to file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to capture visual state of the current page"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "File path to save screenshot (PNG format)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("screenshot.png".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "full_page".to_string(),
                param_type: "boolean".to_string(),
                description: "Capture full page (not just viewport)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_screenshot",
            "parameters": {
                "path": "./screenshot.png"
            }
        })
    }

    fn example_output(&self) -> String {
        "Screenshot saved to ./screenshot.png".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::HaveHeadBrowser
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;

        let full_page = parameters
            .get("full_page")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let tab = get_current_tab()?;

        let png_data = if full_page {
            tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)
                .map_err(|e| anyhow::anyhow!("Failed to capture full page screenshot: {}", e))?
        } else {
            tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, false)
                .map_err(|e| anyhow::anyhow!("Failed to capture viewport screenshot: {}", e))?
        };

        std::fs::write(path, png_data)
            .map_err(|e| anyhow::anyhow!("Failed to save screenshot: {}", e))?;

        Ok(format!("Screenshot saved to {}", path))
    }
}
