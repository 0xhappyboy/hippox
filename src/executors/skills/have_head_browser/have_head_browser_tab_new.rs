//! Browser new tab skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::*;
use crate::executors::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct HaveHeadBrowserTabNewSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserTabNewSkill {
    fn name(&self) -> &str {
        "have_head_browser_tab_new"
    }

    fn description(&self) -> &str {
        "Open a new browser tab"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to open a new tab without closing the current one"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "url".to_string(),
            param_type: "string".to_string(),
            description: "URL to open in the new tab (optional)".to_string(),
            required: false,
            default: None,
            example: Some(Value::String("about:blank".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_tab_new",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        "Opened new tab".to_string()
    }

    fn category(&self) -> &str {
        "have_head_browser"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let browser = get_or_create_browser()?;
        let new_tab = browser
            .new_tab()
            .map_err(|e| anyhow::anyhow!("Failed to create new tab: {}", e))?;
        if let Some(url) = parameters.get("url").and_then(|v| v.as_str()) {
            new_tab
                .navigate_to(url)
                .map_err(|e| anyhow::anyhow!("Failed to navigate: {}", e))?;
            new_tab
                .wait_until_navigated()
                .map_err(|e| anyhow::anyhow!("Navigation timeout: {}", e))?;
        }
        set_current_tab(new_tab);
        Ok("Opened new tab".to_string())
    }
}
