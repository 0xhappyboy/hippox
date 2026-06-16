//! Password verification skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{argon2_verify, bcrypt_verify};
use crate::types::{Skill, SkillParameter};

/// Skill for verifying passwords
///
/// # Description
/// Verifies a password against a stored hash.
///
/// # Parameters
/// * `password` (required) - Password to verify
/// * `hash` (required) - Stored password hash
///
/// # Example
/// ```
/// Input: password="MySecureP@ssw0rd", hash="$2b$12$..."
/// Output: "Password matches"
/// ```
#[derive(Debug)]
pub struct PasswordVerifySkill;

#[async_trait::async_trait]
impl Skill for PasswordVerifySkill {
    fn name(&self) -> &str {
        "password_verify"
    }

    fn description(&self) -> &str {
        "Verify a password against a stored hash"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to verify passwords during authentication. Supports bcrypt and Argon2id hashes."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Password to verify".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("MySecureP@ssw0rd".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "hash".to_string(),
                param_type: "string".to_string(),
                description: "Stored password hash".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("$2b$12$abc123def456...".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "password_verify",
            "parameters": {
                "password": "MySecureP@ssw0rd",
                "hash": "$2b$12$abc123def456..."
            }
        })
    }

    fn example_output(&self) -> String {
        "Password matches".to_string()
    }

    fn category(&self) -> crate::SkillCategory {
        crate::SkillCategory::Cryptography
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let password = parameters
            .get("password")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'password' parameter"))?;
        let hash = parameters
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'hash' parameter"))?;

        // Detect algorithm from hash format
        let is_valid = if hash.starts_with("$2") {
            // bcrypt hash
            bcrypt_verify(password, hash)?
        } else if hash.starts_with("$argon2id") {
            // Argon2id hash
            argon2_verify(password, hash)?
        } else {
            anyhow::bail!("Unsupported hash format: {}", hash)
        };

        if is_valid {
            Ok("Password matches".to_string())
        } else {
            Ok("Password does not match".to_string())
        }
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("password")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: password"))?;
        parameters
            .get("hash")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: hash"))?;
        Ok(())
    }
}
