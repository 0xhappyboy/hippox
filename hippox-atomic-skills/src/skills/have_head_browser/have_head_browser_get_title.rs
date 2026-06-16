//! Browser get page title skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::*;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct HaveHeadBrowserGetTitleSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserGetTitleSkill {
    fn name(&self) -> &str {
        "have_head_browser_get_title"
    }

    fn description(&self) -> &str {
        "Get the current page title"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the title of the current page"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_get_title"
        })
    }

    fn example_output(&self) -> String {
        "Page title: Google".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::HaveHeadBrowser
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let tab = get_current_tab()?;
        let title = tab.get_title().unwrap_or_else(|_| "Unknown".to_string());
        Ok(format!("Page title: {}", title))
    }
}
