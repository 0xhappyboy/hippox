// display_control/display_control_orientation_set.rs
//! Display orientation set skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::SkillCallback;
use crate::SkillContext;
use super::common::set_orientation;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct DisplayControlOrientationSetSkill;

#[async_trait::async_trait]
impl Skill for DisplayControlOrientationSetSkill {
    fn name(&self) -> &str {
        "display_control_orientation_set"
    }

    fn description(&self) -> &str {
        "Set the display orientation"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to rotate the screen orientation (landscape, portrait, etc.)."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "orientation".to_string(),
            param_type: "string".to_string(),
            description:
                "Orientation: 'landscape', 'portrait', 'landscape_flipped', or 'portrait_flipped'"
                    .to_string(),
            required: true,
            default: None,
            example: Some(Value::String("portrait".to_string())),
            enum_values: Some(vec![
                "landscape".to_string(),
                "portrait".to_string(),
                "landscape_flipped".to_string(),
                "portrait_flipped".to_string(),
            ]),
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_orientation_set",
            "parameters": {
                "orientation": "portrait"
            }
        })
    }

    fn example_output(&self) -> String {
        "Display orientation set to portrait".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Display
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let orientation = parameters
            .get("orientation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'orientation' parameter"))?;

        set_orientation(orientation, None)?;

        Ok(format!("Display orientation set to {}", orientation))
    }
}
