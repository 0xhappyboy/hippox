//! Browser get URL skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::*;
use crate::executors::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct HaveHeadBrowserGetUrlSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserGetUrlSkill {
    fn name(&self) -> &str {
        "have_head_browser_get_url"
    }

    fn description(&self) -> &str {
        "Get the current page URL"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check what page the browser is currently on"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
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

    fn category(&self) -> &str {
        "have_head_browser"
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let tab = get_current_tab()?;
        let url = tab.get_url();
        Ok(format!("Current URL: {}", url))
    }
}
