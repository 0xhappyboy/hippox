//! Browser click skill - click element by selector

use crate::{DriverCallback, DriverContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::*;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};

#[derive(Debug)]
pub struct HaveHeadBrowserClickDriver;

#[async_trait::async_trait]
impl Driver for HaveHeadBrowserClickDriver {
    fn name(&self) -> &str {
        "have_head_browser_click"
    }

    fn description(&self) -> &str {
        "Click an element on the current page by CSS selector"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to click buttons, links, or any clickable element. The browser window is visible so user can watch."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description: "CSS selector of the element to click".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("#submit-button".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "wait_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Milliseconds to wait after click (default: 1000)".to_string(),
                required: false,
                default: Some(Value::Number(1000.into())),
                example: Some(Value::Number(500.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_click",
            "parameters": {
                "selector": "#search-button"
            }
        })
    }

    fn example_output(&self) -> String {
        "Clicked element: #search-button".to_string()
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

        let wait_ms = parameters
            .get("wait_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000);

        let tab = get_current_tab()?;
        let element = tab
            .find_element(selector)
            .map_err(|e| anyhow::anyhow!("Element not found: '{}' - {}", selector, e))?;

        element
            .click()
            .map_err(|e| anyhow::anyhow!("Failed to click element '{}': {}", selector, e))?;

        wait_for_stable(&tab, wait_ms).await;

        Ok(format!("Clicked element: {}", selector))
    }
}
