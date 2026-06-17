//! SHA512 hash skill for text

use crate::SkillCallback;
use crate::SkillContext;
use crate::types::{Skill, SkillParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Skill for calculating SHA512 hash of a string
///
/// # Description
/// Computes the SHA512 hash (512-bit) of a given input string and returns it as a hexadecimal string.
/// SHA512 provides stronger security than SHA256 with a larger output size, suitable for
/// applications requiring higher security margins.
///
/// # Parameters
/// * `input` (required) - The string to hash
///
/// # Example
/// ```
/// Input: "Hello World"
/// Output: "SHA512: 2c74fd17edafd80e8447b0d46741ee243b7eb74dd2149a0ab1b9246fb30382f27e853d8585719e0e67cbda0daa8f51671064615d645ae27acb15bfb1447f459b"
/// ```
#[derive(Debug)]
pub struct HashSha512TextSkill;

#[async_trait::async_trait]
impl Skill for HashSha512TextSkill {
    fn name(&self) -> &str {
        "hash_sha512_text"
    }

    fn description(&self) -> &str {
        "Calculate SHA512 hash of a text string"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to compute SHA512 hash for a text string. For file hashing, use file/hash_sha512."
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
            "action": "hash_sha512_text",
            "parameters": {
                "input": "Hello World"
            }
        })
    }

    fn example_output(&self) -> String {
        "SHA512: 2c74fd17edafd80e8447b0d46741ee243b7eb74dd2149a0ab1b9246fb30382f27e853d8585719e0e67cbda0daa8f51671064615d645ae27acb15bfb1447f459b".to_string()
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

        use sha2::{Digest, Sha512};
        let mut hasher = Sha512::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        Ok(format!("SHA512: {}", hex::encode(result)))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("input")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: input"))?;
        Ok(())
    }
}
