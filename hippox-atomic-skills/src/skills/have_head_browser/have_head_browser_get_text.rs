//! Browser get text skill - extract text from elements

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::*;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct HaveHeadBrowserGetTextSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserGetTextSkill {
    fn name(&self) -> &str {
        "have_head_browser_get_text"
    }

    fn description(&self) -> &str {
        "Get text content from an element on the current page"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to extract text from elements like paragraphs, headings, or any element with text"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "CSS selector of the element".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("h1".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "all".to_string(),
                param_type: "boolean".to_string(),
                description: "Get text from all matching elements (default: false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_get_text",
            "parameters": {
                "selector": ".result-title"
            }
        })
    }

    fn example_output(&self) -> String {
        "Text: Example Result Title".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::HaveHeadBrowser
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let selector = parameters
            .get("selector")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: selector"))?;
        let get_all = parameters
            .get("all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let tab = get_current_tab()?;
        if get_all {
            let js = format!(
                r#"
                (function() {{
                    const elements = document.querySelectorAll('{}');
                    return Array.from(elements).map(el => el.innerText || el.textContent || '');
                }})()
                "#,
                selector
            );
            match tab.evaluate(&js, false) {
                Ok(result) => {
                    if let Some(value) = result.value {
                        if let Some(arr) = value.as_array() {
                            let texts: Vec<String> = arr
                                .iter()
                                .enumerate()
                                .filter_map(|(i, v)| {
                                    let text = v.to_string();
                                    if !text.is_empty() && text != "null" {
                                        Some(format!("[{}]: {}", i, text))
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if texts.is_empty() {
                                Ok("No text found in matching elements".to_string())
                            } else {
                                Ok(format!(
                                    "Found {} elements:\n{}",
                                    texts.len(),
                                    texts.join("\n")
                                ))
                            }
                        } else {
                            Ok(format!("Text: {}", value.to_string()))
                        }
                    } else {
                        Ok("No text found".to_string())
                    }
                }
                Err(e) => {
                    anyhow::bail!("Failed to get text: {}", e)
                }
            }
        } else {
            let js = format!(
                r#"
                (function() {{
                    const el = document.querySelector('{}');
                    return el ? (el.innerText || el.textContent || '') : '';
                }})()
                "#,
                selector
            );
            match tab.evaluate(&js, false) {
                Ok(result) => {
                    let text = result
                        .value
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "".to_string());

                    if text.is_empty() || text == "null" {
                        Ok("No text found".to_string())
                    } else {
                        Ok(format!("Text: {}", text))
                    }
                }
                Err(e) => {
                    anyhow::bail!("Element not found or failed to get text: {}", e)
                }
            }
        }
    }
}
