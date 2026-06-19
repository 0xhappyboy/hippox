//! Browser find element skill

use super::shared::*;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct HaveHeadBrowserFindElementDriver;

#[async_trait::async_trait]
impl Driver for HaveHeadBrowserFindElementDriver {
    fn name(&self) -> &str {
        "have_head_browser_find_element"
    }

    fn description(&self) -> &str {
        "Find an element on the current page by CSS selector"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check if an element exists or get its properties before interacting"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "CSS selector of the element to find".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("#submit-button".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "get_text".to_string(),
                param_type: "boolean".to_string(),
                description: "Also return the element's text content (default: false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_find_element",
            "parameters": {
                "selector": ".result-title"
            }
        })
    }

    fn example_output(&self) -> String {
        "Element found: .result-title".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::HaveHeadBrowser
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let selector = parameters
            .get("selector")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: selector"))?;
        let get_text = parameters
            .get("get_text")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let tab = get_current_tab()?;
        match tab.find_element(selector) {
            Ok(element) => {
                let mut result = format!("Element found: {}", selector);
                if get_text {
                    let js = format!(
                        r#"
                        (function() {{
                            const el = document.querySelector('{}');
                            return el ? el.innerText || el.textContent || '' : '';
                        }})()
                        "#,
                        selector
                    );
                    if let Ok(eval_result) = tab.evaluate(&js, false) {
                        if let Some(text) = eval_result.value {
                            let text_str = text.to_string();
                            if !text_str.is_empty() && text_str != "null" {
                                result.push_str(&format!("\nText: {}", text_str));
                            }
                        }
                    }
                }
                Ok(result)
            }
            Err(e) => {
                anyhow::bail!("Element not found: {} - {}", selector, e)
            }
        }
    }
}
