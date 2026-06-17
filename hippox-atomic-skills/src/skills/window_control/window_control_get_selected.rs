//! Window get selected text skill

use crate::SkillCallback;
use crate::SkillCategory;
use crate::SkillContext;
use crate::skills::window_control::WindowControlSendShortcutSkill;
use crate::types::{Skill, SkillParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct WindowControlGetSelectedSkill;

#[async_trait::async_trait]
impl Skill for WindowControlGetSelectedSkill {
    fn name(&self) -> &str {
        "window_control_get_selected"
    }

    fn description(&self) -> &str {
        "Get the currently selected text in the active window"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get text that the user has selected"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "window_control_get_selected"
        })
    }

    fn example_output(&self) -> String {
        "Selected text: Hello World".to_string()
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
        use crate::skills::operating_system::clipboard::ClipboardGetSkill;
        // First copy selected text
        #[cfg(target_os = "windows")]
        {
            let mut params = HashMap::new();
            params.insert("shortcut".to_string(), Value::String("Ctrl+C".to_string()));
            let shortcut_skill = WindowControlSendShortcutSkill;
            let _ = shortcut_skill.execute(&params, callback, context).await;
        }
        #[cfg(not(target_os = "windows"))]
        {
            // Implement for other platforms
        }
        // Then get clipboard content
        let get_skill = ClipboardGetSkill;
        let result = get_skill
            .execute(&HashMap::new(), callback, context)
            .await?;
        Ok(format!("Selected text: {}", result))
    }
}
