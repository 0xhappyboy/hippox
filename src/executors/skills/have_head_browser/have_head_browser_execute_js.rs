//! Browser execute JavaScript skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::shared::*;
use crate::executors::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct HaveHeadBrowserExecuteJsSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserExecuteJsSkill {
    fn name(&self) -> &str {
        "have_head_browser_execute_js"
    }

    fn description(&self) -> &str {
        "Execute JavaScript code in the current page"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to interact with page JavaScript, modify DOM, or extract complex data"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "code".to_string(),
                param_type: "string".to_string(),
                description: "JavaScript code to execute".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("document.title".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "return_value".to_string(),
                param_type: "boolean".to_string(),
                description: "Return the result of the code (default: true)".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_execute_js",
            "parameters": {
                "code": "return document.querySelectorAll('a').length;"
            }
        })
    }

    fn example_output(&self) -> String {
        "Result: 42".to_string()
    }

    fn category(&self) -> &str {
        "have_head_browser"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let code = parameters
            .get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: code"))?;

        let return_value = parameters
            .get("return_value")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let tab = get_current_tab()?;

        if return_value {
            let result = tab
                .evaluate(code, false)
                .map_err(|e| anyhow::anyhow!("Failed to execute JS: {}", e))?;

            let value = result
                .value
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".to_string());

            Ok(format!("Result: {}", value))
        } else {
            tab.evaluate(code, false)
                .map_err(|e| anyhow::anyhow!("Failed to execute JS: {}", e))?;
            Ok("JavaScript executed successfully".to_string())
        }
    }
}
