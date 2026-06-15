//! Browser element exists check skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::*;
use crate::executors::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct HaveHeadBrowserElementExistsSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserElementExistsSkill {
    fn name(&self) -> &str {
        "have_head_browser_element_exists"
    }

    fn description(&self) -> &str {
        "Check if an element exists on the current page"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to verify if an element is present before interacting with it"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "selector".to_string(),
            param_type: "string".to_string(),
            description: "CSS selector to check".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("#submit-button".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_element_exists",
            "parameters": {
                "selector": ".loading-spinner"
            }
        })
    }

    fn example_output(&self) -> String {
        "Element exists: true".to_string()
    }

    fn category(&self) -> &str {
        "have_head_browser"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let selector = parameters
            .get("selector")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: selector"))?;

        let tab = get_current_tab()?;

        let exists = tab.find_element(selector).is_ok();

        Ok(format!("Element exists: {}", exists))
    }
}
