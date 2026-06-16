//! MD5 hash skill for text

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::types::{Skill, SkillParameter};

/// Skill for calculating MD5 hash of a string
///
/// # Description
/// Computes the MD5 hash (128-bit) of a given input string and returns it as a hexadecimal string.
/// MD5 is commonly used for checksums and integrity verification, but is not cryptographically secure.
///
/// # Parameters
/// * `input` (required) - The string to hash
///
/// # Example
/// ```
/// Input: "Hello World"
/// Output: "MD5: b10a8db164e0754105b7a99be72e3fe5"
/// ```
#[derive(Debug)]
pub struct HashMd5TextSkill;

#[async_trait::async_trait]
impl Skill for HashMd5TextSkill {
    fn name(&self) -> &str {
        "hash_md5_text"
    }

    fn description(&self) -> &str {
        "Calculate MD5 hash of a text string"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to compute MD5 hash for a text string. For file hashing, use file/hash_md5."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "input".to_string(),
            param_type: "string".to_string(),
            description: "Input string to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Hello World".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "hash_md5_text",
            "parameters": {
                "input": "Hello World"
            }
        })
    }

    fn example_output(&self) -> String {
        "MD5: b10a8db164e0754105b7a99be72e3fe5".to_string()
    }

    fn category(&self) -> crate::SkillCategory {
        crate::SkillCategory::Cryptography
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let input = parameters
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'input' parameter"))?;

        let digest = md5::compute(input.as_bytes());
        Ok(format!("MD5: {:x}", digest))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("input")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: input"))?;
        Ok(())
    }
}
