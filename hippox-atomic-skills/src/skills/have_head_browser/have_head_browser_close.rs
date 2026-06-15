//! Browser close skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::*;
use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct HaveHeadBrowserCloseSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserCloseSkill {
    fn name(&self) -> &str {
        "have_head_browser_close"
    }

    fn description(&self) -> &str {
        "Close the browser window completely"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to close the browser when no longer needed"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
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

    fn category(&self) -> &str {
        "have_head_browser"
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        close_browser()?;
        Ok("Browser closed".to_string())
    }
}
