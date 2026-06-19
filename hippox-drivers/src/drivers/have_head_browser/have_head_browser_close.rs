//! Browser close skill
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
pub struct HaveHeadBrowserCloseDriver;

#[async_trait::async_trait]
impl Driver for HaveHeadBrowserCloseDriver {
    fn name(&self) -> &str {
        "have_head_browser_close"
    }

    fn description(&self) -> &str {
        "Close the browser window completely"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to close the browser when no longer needed"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_close"
        })
    }

    fn example_output(&self) -> String {
        "Browser closed".to_string()
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
        close_browser()?;
        Ok("Browser closed".to_string())
    }
}
