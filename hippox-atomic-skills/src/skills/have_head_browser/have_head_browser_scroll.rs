//! Browser scroll skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::*;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct HaveHeadBrowserScrollSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserScrollSkill {
    fn name(&self) -> &str {
        "have_head_browser_scroll"
    }

    fn description(&self) -> &str {
        "Scroll the page to a specified position or to an element"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to scroll the page to bring elements into view"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "x".to_string(),
                param_type: "integer".to_string(),
                description: "Horizontal scroll position in pixels".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "y".to_string(),
                param_type: "integer".to_string(),
                description: "Vertical scroll position in pixels".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "CSS selector of element to scroll to (overrides x/y)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("#footer".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "behavior".to_string(),
                param_type: "string".to_string(),
                description: "Scroll behavior: auto or smooth (default: auto)".to_string(),
                required: false,
                default: Some(Value::String("auto".to_string())),
                example: Some(Value::String("smooth".to_string())),
                enum_values: Some(vec!["auto".to_string(), "smooth".to_string()]),
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_scroll",
            "parameters": {
                "y": 1000
            }
        })
    }

    fn example_output(&self) -> String {
        "Scrolled to (0, 1000)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::HaveHeadBrowser
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let tab = get_current_tab()?;
        let behavior = parameters
            .get("behavior")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        if let Some(selector) = parameters.get("selector").and_then(|v| v.as_str()) {
            let js = format!(
                r#"
                const element = document.querySelector('{}');
                if (element) {{
                    element.scrollIntoView({{ behavior: '{}' }});
                    true;
                }} else {{
                    false;
                }}
                "#,
                selector, behavior
            );

            let result = tab
                .evaluate(&js, false)
                .map_err(|e| anyhow::anyhow!("Failed to scroll to element: {}", e))?;

            let found = result.value.and_then(|v| v.as_bool()).unwrap_or(false);

            if found {
                Ok(format!("Scrolled to element: {}", selector))
            } else {
                anyhow::bail!("Element not found: {}", selector)
            }
        } else {
            let x = parameters.get("x").and_then(|v| v.as_u64()).unwrap_or(0);
            let y = parameters.get("y").and_then(|v| v.as_u64()).unwrap_or(0);

            let js = format!(
                "window.scrollTo({{ left: {}, top: {}, behavior: '{}' }});",
                x, y, behavior
            );

            tab.evaluate(&js, false)
                .map_err(|e| anyhow::anyhow!("Failed to scroll: {}", e))?;

            Ok(format!("Scrolled to ({}, {})", x, y))
        }
    }
}
