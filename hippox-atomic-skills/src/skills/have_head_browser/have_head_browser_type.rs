//! Browser type skill - type text into input field

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::*;
use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct HaveHeadBrowserTypeSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserTypeSkill {
    fn name(&self) -> &str {
        "have_head_browser_type"
    }

    fn description(&self) -> &str {
        "Type text into an input field on the current page"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to fill input fields, textareas, or search boxes. First click the field if needed."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "CSS selector of the input element".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("#search-input".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "text".to_string(),
                param_type: "string".to_string(),
                description: "Text to type into the input field".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello World".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "clear_first".to_string(),
                param_type: "boolean".to_string(),
                description: "Clear existing text before typing (default: true)".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_type",
            "parameters": {
                "selector": "#search-input",
                "text": "Rust programming"
            }
        })
    }

    fn example_output(&self) -> String {
        "Typed 'Rust programming' into #search-input".to_string()
    }

    fn category(&self) -> &str {
        "have_head_browser"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let selector = parameters
            .get("selector")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: selector"))?;
        let text = parameters
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: text"))?;
        let clear_first = parameters
            .get("clear_first")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let tab = get_current_tab()?;
        let element = tab
            .find_element(selector)
            .map_err(|e| anyhow::anyhow!("Element not found: '{}' - {}", selector, e))?;

        if clear_first {
            element
                .click()
                .map_err(|e| anyhow::anyhow!("Failed to click element: {}", e))?;
        }
        let js = if clear_first {
            format!(
                r#"
                (function() {{
                    const el = document.querySelector('{}');
                    if (el) {{
                        el.value = '';
                        el.value = {};
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        return true;
                    }}
                    return false;
                }})()
                "#,
                selector,
                serde_json::to_string(text)?
            )
        } else {
            format!(
                r#"
                (function() {{
                    const el = document.querySelector('{}');
                    if (el) {{
                        el.value = {};
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        return true;
                    }}
                    return false;
                }})()
                "#,
                selector,
                serde_json::to_string(text)?
            )
        };
        let result = tab
            .evaluate(&js, false)
            .map_err(|e| anyhow::anyhow!("Failed to type text: {}", e))?;
        if !result.value.and_then(|v| v.as_bool()).unwrap_or(false) {
            anyhow::bail!("Element not found: {}", selector);
        }
        Ok(format!("Typed '{}' into {}", text, selector))
    }
}
