//! Browser close tab skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::*;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct HaveHeadBrowserTabCloseSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserTabCloseSkill {
    fn name(&self) -> &str {
        "have_head_browser_tab_close"
    }

    fn description(&self) -> &str {
        "Close the current browser tab"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to close the current tab"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_tab_close"
        })
    }

    fn example_output(&self) -> String {
        "Closed current tab".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::HaveHeadBrowser
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let tab = get_current_tab()?;
        tab.close(false)
            .map_err(|e| anyhow::anyhow!("Failed to close tab: {}", e))?;
        clear_current_tab();
        Ok("Closed current tab".to_string())
    }
}
