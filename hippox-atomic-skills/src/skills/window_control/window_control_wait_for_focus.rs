//! Window wait for focus skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::common::{find_window, get_focus_window};
use crate::{SkillCallback, SkillCategory, SkillContext};
use crate::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct WindowControlWaitForFocusSkill;

#[async_trait::async_trait]
impl Skill for WindowControlWaitForFocusSkill {
    fn name(&self) -> &str {
        "window_control_wait_for_focus"
    }

    fn description(&self) -> &str {
        "Wait for a window to gain focus"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to wait until a specific window becomes active"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Window title to wait for".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("微信".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "process".to_string(),
                param_type: "string".to_string(),
                description: "Process name to wait for".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("WeChat.exe".to_string())),
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_wait_for_focus",
            "parameters": {
                "title": "微信",
                "timeout_ms": 10000
            }
        })
    }

    fn example_output(&self) -> String {
        "Window gained focus after 1234ms".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Window
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let title = parameters.get("title").and_then(|v| v.as_str());
        let process = parameters.get("process").and_then(|v| v.as_str());
        let timeout_ms = parameters
            .get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(30000);
        let start = Instant::now();
        loop {
            if start.elapsed() > Duration::from_millis(timeout_ms) {
                anyhow::bail!("Timeout waiting for window to gain focus");
            }
            let focused_id = get_focus_window()?;
            use super::common::list_windows;
            let windows = list_windows()?;
            if let Some(focused) = windows.iter().find(|w| w.id == focused_id) {
                let match_title = title.map_or(false, |t| {
                    focused.title.to_lowercase().contains(&t.to_lowercase())
                });
                let match_process = process.map_or(false, |p| {
                    focused
                        .process_name
                        .to_lowercase()
                        .contains(&p.to_lowercase())
                });

                if match_title || match_process {
                    return Ok(format!(
                        "Window gained focus after {}ms",
                        start.elapsed().as_millis()
                    ));
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
