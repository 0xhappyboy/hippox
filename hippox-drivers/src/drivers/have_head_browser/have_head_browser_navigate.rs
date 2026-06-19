//! Browser navigation skill - navigate to URL in visible browser window

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
pub struct HaveHeadBrowserNavigateDriver;

#[async_trait::async_trait]
impl Driver for HaveHeadBrowserNavigateDriver {
    fn name(&self) -> &str {
        "have_head_browser_navigate"
    }

    fn description(&self) -> &str {
        "Navigate to a URL in the visible browser window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to open a web page in the browser. A visible browser window will pop up."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "url".to_string(),
                param_type: "string".to_string(),
                description: "URL to navigate to (e.g., https://example.com)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("https://www.google.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "wait_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Milliseconds to wait after navigation (default: 2000)".to_string(),
                required: false,
                default: Some(Value::Number(2000.into())),
                example: Some(Value::Number(3000.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_navigate",
            "parameters": {
                "url": "https://www.google.com"
            }
        })
    }

    fn example_output(&self) -> String {
        "Navigated to https://www.google.com".to_string()
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
        let url = parameters
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: url"))?;

        let wait_ms = parameters
            .get("wait_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(2000);

        let tab = get_current_tab()?;

        tab.navigate_to(url)
            .map_err(|e| anyhow::anyhow!("Failed to navigate: {}", e))?;

        tab.wait_until_navigated()
            .map_err(|e| anyhow::anyhow!("Navigation timeout: {}", e))?;

        wait_for_stable(&tab, wait_ms).await;

        Ok(format!("Navigated to {}", url))
    }
}
