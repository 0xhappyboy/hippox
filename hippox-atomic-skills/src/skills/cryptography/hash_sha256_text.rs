//! SHA256 hash skill for text

use crate::SkillCallback;
use crate::SkillContext;
use crate::types::{Skill, SkillParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Skill for calculating SHA256 hash of a string
///
/// # Description
/// Computes the SHA256 hash (256-bit) of a given input string and returns it as a hexadecimal string.
/// SHA256 is a cryptographically secure hash function commonly used in security applications,
/// digital signatures, and blockchain technologies.
///
/// # Parameters
/// * `input` (required) - The string to hash
///
/// # Example
/// ```
/// Input: "Hello World"
/// Output: "SHA256: a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e"
/// ```
#[derive(Debug)]
pub struct HashSha256TextSkill;

#[async_trait::async_trait]
impl Skill for HashSha256TextSkill {
    fn name(&self) -> &str {
        "hash_sha256_text"
    }

    fn description(&self) -> &str {
        "Calculate SHA256 hash of a text string"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to compute SHA256 hash for a text string. For file hashing, use file/hash_sha256."
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
            "action": "hash_sha256_text",
            "parameters": {
                "input": "Hello World"
            }
        })
    }

    fn example_output(&self) -> String {
        "SHA256: a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e".to_string()
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
        let input = parameters
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'input' parameter"))?;

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        Ok(format!("SHA256: {}", hex::encode(result)))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("input")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: input"))?;
        Ok(())
    }
}
