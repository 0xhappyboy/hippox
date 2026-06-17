//! UUID generation skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{SkillCallback, SkillContext, types::{Skill, SkillParameter}};

/// Skill for generating UUID
///
/// # Description
/// Generates a universally unique identifier (UUID) version 4 (random).
///
/// # Parameters
/// * `format` (optional) - "hyphenated" (default), "simple", "braced", or "urn"
///
/// # Example
/// ```
/// Input: format="hyphenated"
/// Output: "UUID: 550e8400-e29b-41d4-a716-446655440000"
/// ```
#[derive(Debug)]
pub struct GenerateUuidSkill;

#[async_trait::async_trait]
impl Skill for GenerateUuidSkill {
    fn name(&self) -> &str {
        "generate_uuid"
    }

    fn description(&self) -> &str {
        "Generate a UUID (version 4)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need a unique identifier for resources, sessions, or tracking."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "format".to_string(),
                param_type: "string".to_string(),
                description: "Output format: 'hyphenated', 'simple', 'braced', or 'urn'"
                    .to_string(),
                required: false,
                default: Some(Value::String("hyphenated".to_string())),
                example: Some(Value::String("simple".to_string())),
                enum_values: Some(vec![
                    "hyphenated".to_string(),
                    "simple".to_string(),
                    "braced".to_string(),
                    "urn".to_string(),
                ]),
            },
            SkillParameter {
                name: "count".to_string(),
                param_type: "integer".to_string(),
                description: "Number of UUIDs to generate (default: 1)".to_string(),
                required: false,
                default: Some(Value::Number(1.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "generate_uuid",
            "parameters": {
                "format": "hyphenated"
            }
        })
    }

    fn example_output(&self) -> String {
        "UUID: 550e8400-e29b-41d4-a716-446655440000".to_string()
    }

    fn category(&self) -> crate::SkillCategory {
        crate::SkillCategory::Cryptography
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let format = parameters
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("hyphenated");
        let count = parameters
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;

        if count == 0 {
            anyhow::bail!("Count must be greater than 0");
        }

        let mut uuids = Vec::new();
        for _ in 0..count {
            let uuid = Uuid::new_v4();
            let formatted = match format {
                "simple" => uuid.simple().to_string(),
                "braced" => uuid.braced().to_string(),
                "urn" => uuid.urn().to_string(),
                _ => uuid.hyphenated().to_string(),
            };
            uuids.push(formatted);
        }

        if uuids.len() == 1 {
            Ok(format!("UUID: {}", uuids[0]))
        } else {
            let mut output = String::from("UUIDs:\n");
            for (i, uuid) in uuids.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", i + 1, uuid));
            }
            Ok(output)
        }
    }

    fn validate(&self, _parameters: &HashMap<String, Value>) -> Result<()> {
        Ok(())
    }
}
