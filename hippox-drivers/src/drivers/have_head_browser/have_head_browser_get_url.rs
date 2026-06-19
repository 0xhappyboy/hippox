//! Browser get URL skill

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
pub struct HaveHeadBrowserGetUrlDriver;

#[async_trait::async_trait]
impl Driver for HaveHeadBrowserGetUrlDriver {
    fn name(&self) -> &str {
        "have_head_browser_get_url"
    }

    fn description(&self) -> &str {
        "Get the current page URL"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check what page the browser is currently on"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_get_url"
        })
    }

    fn example_output(&self) -> String {
        "Current URL: https://www.google.com".to_string()
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
        let tab = get_current_tab()?;
        let url = tab.get_url();
        Ok(format!("Current URL: {}", url))
    }
}
