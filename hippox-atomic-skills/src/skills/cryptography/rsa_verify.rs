//! RSA signature verification skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{from_base64, rsa_verify};
use crate::{
    types::{Skill, SkillParameter},
};

/// Skill for RSA signature verification
///
/// # Description
/// Verifies a digital signature using RSA public key.
///
/// # Parameters
/// * `public_key` (required) - RSA public key in PEM format
/// * `data` (required) - Original data that was signed (string)
/// * `signature` (required) - Base64 signature to verify
///
/// # Example
/// ```
/// Input: public_key="-----BEGIN PUBLIC KEY-----...", data="Hello World", signature="7f83b1657ff1fc53..."
/// Output: "Signature is valid"
/// ```
#[derive(Debug)]
pub struct RsaVerifySkill;

#[async_trait::async_trait]
impl Skill for RsaVerifySkill {
    fn name(&self) -> &str {
        "rsa_verify"
    }

    fn description(&self) -> &str {
        "Verify RSA digital signature using public key"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to verify an RSA signature. Provide the public key, original data, and the signature."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "public_key".to_string(),
                param_type: "string".to_string(),
                description: "RSA public key in PEM format".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("-----BEGIN PUBLIC KEY-----\n...".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "data".to_string(),
                param_type: "string".to_string(),
                description: "Original data that was signed".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello World".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "signature".to_string(),
                param_type: "string".to_string(),
                description: "Base64 signature to verify".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("7f83b1657ff1fc53...".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "rsa_verify",
            "parameters": {
                "public_key": "-----BEGIN PUBLIC KEY-----\n...",
                "data": "Hello World",
                "signature": "7f83b1657ff1fc53..."
            }
        })
    }

    fn example_output(&self) -> String {
        "Signature is valid".to_string()
    }

    fn category(&self) -> crate::SkillCategory {
        crate::SkillCategory::Cryptography
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let public_key = parameters
            .get("public_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'public_key' parameter"))?;
        let data = parameters
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'data' parameter"))?;
        let signature = parameters
            .get("signature")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'signature' parameter"))?;

        let signature_bytes = from_base64(signature)?;
        let is_valid = rsa_verify(public_key, data.as_bytes(), &signature_bytes)?;

        if is_valid {
            Ok("Signature is valid".to_string())
        } else {
            Ok("Signature is invalid".to_string())
        }
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("public_key")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: public_key"))?;
        parameters
            .get("data")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: data"))?;
        parameters
            .get("signature")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: signature"))?;
        Ok(())
    }
}