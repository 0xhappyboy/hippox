//! Browser back navigation skill

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
pub struct HaveHeadBrowserBackDriver;

#[async_trait::async_trait]
impl Driver for HaveHeadBrowserBackDriver {
    fn name(&self) -> &str {
        "have_head_browser_back"
    }

    fn description(&self) -> &str {
        "Navigate back to the previous page"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to go back to the previous page in history"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_back"
        })
    }

    fn example_output(&self) -> String {
        "Navigated back".to_string()
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

        tab.evaluate("window.history.back()", false)
            .map_err(|e| anyhow::anyhow!("Failed to navigate back: {}", e))?;

        wait_for_stable(&tab, 1000).await;

        Ok("Navigated back".to_string())
    }
}
