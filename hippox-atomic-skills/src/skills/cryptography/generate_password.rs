//! Password generation skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{generate_random_bytes, validate_password_strength};
use crate::types::{Skill, SkillParameter};

/// Skill for generating secure passwords
#[derive(Debug)]
pub struct GeneratePasswordSkill;

#[async_trait::async_trait]
impl Skill for GeneratePasswordSkill {
    fn name(&self) -> &str {
        "generate_password"
    }

    fn description(&self) -> &str {
        "Generate secure random passwords"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to generate secure passwords for users or services."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "length".to_string(),
                param_type: "integer".to_string(),
                description: "Password length (default: 16)".to_string(),
                required: false,
                default: Some(Value::Number(16.into())),
                example: Some(Value::Number(20.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "count".to_string(),
                param_type: "integer".to_string(),
                description: "Number of passwords to generate (default: 1)".to_string(),
                required: false,
                default: Some(Value::Number(1.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "special_chars".to_string(),
                param_type: "string".to_string(),
                description: "Custom special characters (default: !@#$%^&*()_+-=)".to_string(),
                required: false,
                default: Some(Value::String("!@#$%^&*()_+-=".to_string())),
                example: Some(Value::String("!@#$%".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "generate_password",
            "parameters": {
                "length": 16,
                "count": 1
            }
        })
    }

    fn example_output(&self) -> String {
        "Generated password: Kx9#mP2$vL5@nQ8!rT3".to_string()
    }

    fn category(&self) -> crate::SkillCategory {
        crate::SkillCategory::Cryptography
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let length = parameters
            .get("length")
            .and_then(|v| v.as_u64())
            .unwrap_or(16) as usize;
        let count = parameters
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;
        let custom_special = parameters
            .get("special_chars")
            .and_then(|v| v.as_str())
            .unwrap_or("!@#$%^&*()_+-=");
        if length < 8 {
            anyhow::bail!("Password length must be at least 8 characters");
        }
        if count == 0 {
            anyhow::bail!("Count must be greater than 0");
        }
        let mut passwords = Vec::new();
        for _ in 0..count {
            let password = generate_secure_password(length, custom_special)?;
            passwords.push(password);
        }
        if passwords.len() == 1 {
            Ok(format!("Generated password: {}", passwords[0]))
        } else {
            let mut output = String::from("Generated passwords:\n");
            for (i, pwd) in passwords.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", i + 1, pwd));
            }
            Ok(output)
        }
    }

    fn validate(&self, _parameters: &HashMap<String, Value>) -> Result<()> {
        Ok(())
    }
}

/// Generate a secure password with required character types  
fn generate_secure_password(length: usize, special_chars: &str) -> Result<String> {
    let uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let lowercase = "abcdefghijklmnopqrstuvwxyz";
    let digits = "0123456789";
    let all_chars = format!("{}{}{}{}", uppercase, lowercase, digits, special_chars);
    let all_chars_bytes = all_chars.as_bytes();
    let all_chars_len = all_chars_bytes.len();
    let random_bytes = generate_random_bytes(length)?;
    let mut password_chars: Vec<char> = Vec::with_capacity(length);
    for b in random_bytes {
        let idx = (b as usize) % all_chars_len;
        password_chars.push(all_chars_bytes[idx] as char);
    }
    let mut password: String = password_chars.into_iter().collect();
    let mut chars: Vec<char> = password.chars().collect();
    if !chars.iter().any(|c| c.is_uppercase()) {
        let idx = rng_index(length);
        chars[idx] = uppercase.chars().nth(rng_index(uppercase.len())).unwrap();
    }
    if !chars.iter().any(|c| c.is_lowercase()) {
        let idx = rng_index(length);
        chars[idx] = lowercase.chars().nth(rng_index(lowercase.len())).unwrap();
    }
    if !chars.iter().any(|c| c.is_ascii_digit()) {
        let idx = rng_index(length);
        chars[idx] = digits.chars().nth(rng_index(digits.len())).unwrap();
    }
    if !chars.iter().any(|c| special_chars.contains(*c)) {
        let idx = rng_index(length);
        chars[idx] = special_chars
            .chars()
            .nth(rng_index(special_chars.len()))
            .unwrap();
    }
    let result: String = chars.into_iter().collect();
    validate_password_strength(&result)?;
    Ok(result)
}

fn rng_index(max: usize) -> usize {
    if max == 0 {
        return 0;
    }
    let mut bytes = [0u8; 1];
    let _ = getrandom::fill(&mut bytes);
    (bytes[0] as usize) % max
}
