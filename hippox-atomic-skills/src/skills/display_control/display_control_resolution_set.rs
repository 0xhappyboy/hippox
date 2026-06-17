// display_control/display_control_resolution_set.rs
//! Display resolution set skill

use super::common::set_resolution;
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
pub struct DisplayControlResolutionSetSkill;

#[async_trait::async_trait]
impl Skill for DisplayControlResolutionSetSkill {
    fn name(&self) -> &str {
        "display_control_resolution_set"
    }

    fn description(&self) -> &str {
        "Set the display resolution"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to change the screen resolution. May cause temporary screen flicker."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "width".to_string(),
                param_type: "integer".to_string(),
                description: "Desired width in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(1920.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "height".to_string(),
                param_type: "integer".to_string(),
                description: "Desired height in pixels".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(1080.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "display_id".to_string(),
                param_type: "integer".to_string(),
                description: "Display ID (optional, uses primary if not specified)".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(1.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "display_control_resolution_set",
            "parameters": {
                "width": 1920,
                "height": 1080
            }
        })
    }

    fn example_output(&self) -> String {
        "Resolution set to 1920x1080".to_string()
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
        let width = parameters
            .get("width")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'width' parameter"))?
            as u32;

        let height = parameters
            .get("height")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'height' parameter"))?
            as u32;

        let display_id = parameters
            .get("display_id")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        set_resolution(width, height, display_id)?;

        Ok(format!("Resolution set to {}x{}", width, height))
    }
}
