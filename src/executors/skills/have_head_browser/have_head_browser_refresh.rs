//! Browser refresh/reload skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::*;
use crate::executors::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct HaveHeadBrowserRefreshSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserRefreshSkill {
    fn name(&self) -> &str {
        "have_head_browser_refresh"
    }

    fn description(&self) -> &str {
        "Refresh the current page"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to reload the current page"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
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

    fn category(&self) -> &str {
        "have_head_browser"
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let tab = get_current_tab()?;
        tab.reload(false, None)
            .map_err(|e| anyhow::anyhow!("Failed to refresh: {}", e))?;
        wait_for_stable(&tab, 2000).await;
        Ok("Page refreshed".to_string())
    }
}
