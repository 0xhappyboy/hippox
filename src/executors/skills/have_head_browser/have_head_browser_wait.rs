//! Browser wait skill - wait for time or element

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::shared::*;
use crate::executors::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct HaveHeadBrowserWaitSkill;

#[async_trait::async_trait]
impl Skill for HaveHeadBrowserWaitSkill {
    fn name(&self) -> &str {
        "have_head_browser_wait"
    }

    fn description(&self) -> &str {
        "Wait for a specified time or for an element to appear"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to wait for page content to load or animations to complete"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "selector".to_string(),
                param_type: "string".to_string(),
                description:
                    "CSS selector to wait for (optional, if not provided, waits fixed time)"
                        .to_string(),
                required: false,
                default: None,
                example: Some(Value::String(".loading-done".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum wait time in milliseconds (default: 30000)".to_string(),
                required: false,
                default: Some(Value::Number(30000.into())),
                example: Some(Value::Number(5000.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "wait_ms".to_string(),
                param_type: "integer".to_string(),
                description: "Fixed wait time in milliseconds (used if selector not provided)"
                    .to_string(),
                required: false,
                default: Some(Value::Number(1000.into())),
                example: Some(Value::Number(2000.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "have_head_browser_wait",
            "parameters": {
                "wait_ms": 2000
            }
        })
    }

    fn example_output(&self) -> String {
        "Waited for 2000ms".to_string()
    }

    fn category(&self) -> &str {
        "have_head_browser"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let selector = parameters.get("selector").and_then(|v| v.as_str());
        let timeout_ms = parameters
            .get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(30000);

        if let Some(sel) = selector {
            let tab = get_current_tab()?;
            let start = Instant::now();
            let timeout_dur = Duration::from_millis(timeout_ms);

            loop {
                if start.elapsed() > timeout_dur {
                    anyhow::bail!("Timeout waiting for element: {}", sel);
                }

                match tab.find_element(sel) {
                    Ok(_) => break,
                    Err(_) => {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
            }

            Ok(format!(
                "Element '{}' appeared after {}ms",
                sel,
                start.elapsed().as_millis()
            ))
        } else {
            let wait_ms = parameters
                .get("wait_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(1000);

            tokio::time::sleep(Duration::from_millis(wait_ms)).await;
            Ok(format!("Waited for {}ms", wait_ms))
        }
    }
}
