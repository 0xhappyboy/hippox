//! Base64 decoding skill

use anyhow::Result;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    types::{Skill, SkillParameter},
};

/// Skill for Base64 decoding
///
/// # Description
/// Decodes a Base64 string back to its original text representation. The decoded data must be
/// valid UTF-8; otherwise, an error is returned indicating that the decoded data is not
/// valid UTF-8.
///
/// # Parameters
/// * `input` (required) - The Base64 string to decode
///
/// # Example
/// ```
/// Input: "SGVsbG8gV29ybGQ="
/// Output: "Decoded: Hello World"
/// ```
#[derive(Debug)]
pub struct Base64DecodeSkill;

#[async_trait::async_trait]
impl Skill for Base64DecodeSkill {
    fn name(&self) -> &str {
        "base64_decode"
    }

    fn description(&self) -> &str {
        "Decode a Base64 string to original text"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to decode Base64 encoded text"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "input".to_string(),
            param_type: "string".to_string(),
            description: "Base64 string to decode".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("SGVsbG8gV29ybGQ=".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "base64_decode",
            "parameters": {
                "input": "SGVsbG8gV29ybGQ="
            }
        })
    }

    fn example_output(&self) -> String {
        "Decoded: Hello World".to_string()
    }

    fn category(&self) -> crate::SkillCategory {
        crate::SkillCategory::Cryptography
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let input = parameters
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'input' parameter"))?;
        let decoded = STANDARD
            .decode(input)
            .map_err(|e| anyhow::anyhow!("Invalid Base64 string: {}", e))?;
        let decoded_str = String::from_utf8(decoded)
            .map_err(|e| anyhow::anyhow!("Decoded data is not valid UTF-8: {}", e))?;
        Ok(format!("Decoded: {}", decoded_str))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("input")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: input"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::skills::Base64EncodeSkill;

use super::*;

    #[tokio::test]
    async fn test_base64_encode_decode_roundtrip() {
        let encode_skill = Base64EncodeSkill;
        let decode_skill = Base64DecodeSkill;
        let test_cases = vec![
            "Hello World",
            "Rust programming language",
            "Base64编码测试",
            "1234567890!@#$%^&*()",
            "",
            "a", // Single character
        ];
        for test_input in test_cases {
            let mut encode_params = HashMap::new();
            encode_params.insert("input".to_string(), Value::String(test_input.to_string()));
            // Encode
            let encoded_result = encode_skill.execute(&encode_params).await.unwrap();
            let encoded = encoded_result.trim_start_matches("Base64: ");
            // Decode
            let mut decode_params = HashMap::new();
            decode_params.insert("input".to_string(), Value::String(encoded.to_string()));
            let decoded_result = decode_skill.execute(&decode_params).await.unwrap();
            let decoded = decoded_result.trim_start_matches("Decoded: ");
            // Verify roundtrip
            assert_eq!(test_input, decoded, "Failed for input: {}", test_input);
        }
    }
}