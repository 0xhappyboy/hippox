//! Window get focus skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::get_focus_window;
use crate::{SkillCategory, types::{Skill, SkillParameter}};

#[derive(Debug)]
pub struct WindowControlGetFocusSkill;

#[async_trait::async_trait]
impl Skill for WindowControlGetFocusSkill {
    fn name(&self) -> &str {
        "window_control_get_focus"
    }

    fn description(&self) -> &str {
        "Get the currently focused window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to find out which window is currently active"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_get_focus"
        })
    }

    fn example_output(&self) -> String {
        "Focused window: 微信 (WeChat.exe)".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Window
    }

    async fn execute(&self, _parameters: &HashMap<String, Value>) -> Result<String> {
        let window_id = get_focus_window()?;

        use super::common::list_windows;
        let windows = list_windows()?;
        let window = windows
            .iter()
            .find(|w| w.id == window_id)
            .ok_or_else(|| anyhow::anyhow!("Window not found"))?;

        Ok(format!(
            "Focused window: {} ({})",
            window.title, window.process_name
        ))
    }
}
