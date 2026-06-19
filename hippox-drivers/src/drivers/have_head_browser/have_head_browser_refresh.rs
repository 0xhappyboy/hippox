//! Browser refresh/reload skill

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
pub struct HaveHeadBrowserRefreshDriver;

#[async_trait::async_trait]
impl Driver for HaveHeadBrowserRefreshDriver {
    fn name(&self) -> &str {
        "have_head_browser_refresh"
    }

    fn description(&self) -> &str {
        "Refresh the current page"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to reload the current page"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_refresh"
        })
    }

    fn example_output(&self) -> String {
        "Page refreshed".to_string()
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
        tab.reload(false, None)
            .map_err(|e| anyhow::anyhow!("Failed to refresh: {}", e))?;
        wait_for_stable(&tab, 2000).await;
        Ok("Page refreshed".to_string())
    }
}
