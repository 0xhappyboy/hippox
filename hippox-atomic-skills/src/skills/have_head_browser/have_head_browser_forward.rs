//! Browser forward navigation skill

use super::shared::*;
use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct HaveHeadBrowserForwardSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserForwardSkill {
    fn name(&self) -> &str {
        "have_head_browser_forward"
    }

    fn description(&self) -> &str {
        "Navigate forward to the next page"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to go forward after a back navigation"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_forward"
        })
    }

    fn example_output(&self) -> String {
        "Navigated forward".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::HaveHeadBrowser
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let tab = get_current_tab()?;

        tab.evaluate("window.history.forward()", false)
            .map_err(|e| anyhow::anyhow!("Failed to navigate forward: {}", e))?;

        wait_for_stable(&tab, 1000).await;

        Ok("Navigated forward".to_string())
    }
}
