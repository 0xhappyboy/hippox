//! Browser switch tab skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::*;
use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct HaveHeadBrowserTabSwitchSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserTabSwitchSkill {
    fn name(&self) -> &str {
        "have_head_browser_tab_switch"
    }

    fn description(&self) -> &str {
        "Switch to a different browser tab by index"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to switch between open tabs. Index 0 is the first tab."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "index".to_string(),
            param_type: "integer".to_string(),
            description: "Tab index to switch to (0-based)".to_string(),
            required: true,
            default: None,
            example: Some(Value::Number(0.into())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_tab_switch",
            "parameters": {
                "index": 0
            }
        })
    }

    fn example_output(&self) -> String {
        "Switched to tab 0".to_string()
    }

    fn category(&self) -> &str {
        "have_head_browser"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let index = parameters
            .get("index")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: index (0-based)"))?
            as usize;

        let browser = get_or_create_browser()?;
        let tabs_guard = browser.get_tabs();
        let tabs = tabs_guard.lock().unwrap();

        if index >= tabs.len() {
            anyhow::bail!(
                "Tab index {} out of range ({} tabs available)",
                index,
                tabs.len()
            );
        }
        let target_tab = tabs[index].clone();
        set_current_tab(target_tab);
        Ok(format!("Switched to tab {}", index))
    }
}
