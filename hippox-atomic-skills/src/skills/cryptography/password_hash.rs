//! Password hashing skill

use super::common::{argon2_hash, bcrypt_hash, validate_password_strength};
use crate::SkillCallback;
use crate::SkillContext;
use crate::types::{Skill, SkillParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Skill for hashing passwords
///
/// # Description
/// Hashes passwords using bcrypt or Argon2id for secure storage.
///
/// # Parameters
/// * `password` (required) - Password to hash
/// * `algorithm` (optional) - "bcrypt" (default) or "argon2id"
/// * `cost` (optional) - Cost factor for bcrypt (4-31, default 12)
/// * `validate` (optional) - Validate password strength (default true)
///
/// # Example
/// ```
/// Input: password="MySecureP@ssw0rd", algorithm="bcrypt", cost=12
/// Output: "Hashed: $2b$12$..."
/// ```
#[derive(Debug)]
pub struct PasswordHashSkill;

#[async_trait::async_trait]
impl Skill for PasswordHashSkill {
    fn name(&self) -> &str {
        "password_hash"
    }

    fn description(&self) -> &str {
        "Hash a password using bcrypt or Argon2id"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to securely hash passwords for storage. Supports bcrypt (default) and Argon2id."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Password to hash".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("MySecureP@ssw0rd".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "algorithm".to_string(),
                param_type: "string".to_string(),
                description: "Hash algorithm: 'bcrypt' or 'argon2id'".to_string(),
                required: false,
                default: Some(Value::String("bcrypt".to_string())),
                example: Some(Value::String("argon2id".to_string())),
                enum_values: Some(vec!["bcrypt".to_string(), "argon2id".to_string()]),
            },
            SkillParameter {
                name: "cost".to_string(),
                param_type: "integer".to_string(),
                description: "Cost factor for bcrypt (4-31, default 12)".to_string(),
                required: false,
                default: Some(Value::Number(12.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "validate".to_string(),
                param_type: "boolean".to_string(),
                description: "Validate password strength (default true)".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "password_hash",
            "parameters": {
                "password": "MySecureP@ssw0rd",
                "algorithm": "bcrypt",
                "cost": 12
            }
        })
    }

    fn example_output(&self) -> String {
        "Hashed: $2b$12$abc123def456...".to_string()
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
        let password = parameters
            .get("password")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'password' parameter"))?;
        let algorithm = parameters
            .get("algorithm")
            .and_then(|v| v.as_str())
            .unwrap_or("bcrypt");
        let cost = parameters
            .get("cost")
            .and_then(|v| v.as_u64())
            .unwrap_or(12) as u32;
        let validate = parameters
            .get("validate")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if validate {
            validate_password_strength(password)?;
        }

        let hashed = match algorithm {
            "bcrypt" => {
                if cost < 4 || cost > 31 {
                    anyhow::bail!("Cost must be between 4 and 31");
                }
                bcrypt_hash(password, cost)?
            }
            "argon2id" => argon2_hash(password)?,
            _ => anyhow::bail!("Unsupported algorithm: {}", algorithm),
        };

        Ok(format!("Hashed: {}", hashed))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("password")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: password"))?;
        Ok(())
    }
}
