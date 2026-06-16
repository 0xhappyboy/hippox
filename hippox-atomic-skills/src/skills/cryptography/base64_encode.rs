//! Base64 encoding skill

use anyhow::Result;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    types::{Skill, SkillParameter},
};

/// Skill for Base64 encoding
///
/// # Description
/// Encodes a string to Base64 format. Base64 encoding is commonly used to represent binary data
/// in an ASCII string format, useful for data transmission over text-based protocols like
/// HTTP or email attachments.
///
/// # Parameters
/// * `input` (required) - The string to encode
///
/// # Example
/// ```
/// Input: "Hello World"
/// Output: "Base64: SGVsbG8gV29ybGQ="
/// ```
#[derive(Debug)]
pub struct Base64EncodeSkill;

#[async_trait::async_trait]
impl Skill for Base64EncodeSkill {
    fn name(&self) -> &str {
        "base64_encode"
    }

    fn description(&self) -> &str {
        "Encode a string to Base64"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to convert text to Base64 encoding"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "input".to_string(),
            param_type: "string".to_string(),
            description: "Input string to encode".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Hello World".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "base64_encode",
            "parameters": {
                "input": "Hello World"
            }
        })
    }

    fn example_output(&self) -> String {
        "Base64: SGVsbG8gV29ybGQ=".to_string()
    }

    fn category(&self) -> crate::SkillCategory {
        crate::SkillCategory::Cryptography
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let input = parameters
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'input' parameter"))?;
        let encoded = STANDARD.encode(input.as_bytes());
        Ok(format!("Base64: {}", encoded))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("input")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: input"))?;
        Ok(())
    }
}