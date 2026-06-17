//! Weak password detection skill

use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    operating_system_security::common::{get_password_strength, is_password_weak},
    types::{Skill, SkillParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct WeakPasswordCheckSkill;

#[async_trait::async_trait]
impl Skill for WeakPasswordCheckSkill {
    fn name(&self) -> &str {
        "security_weak_password_check"
    }

    fn description(&self) -> &str {
        "Check if a password is weak or meets security requirements"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to test password strength. Checks against common weak passwords, length, complexity, and patterns."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Password to check".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("MySecureP@ssw0rd123".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "Associated username (optional, for context)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("admin".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "security_weak_password_check",
            "parameters": {
                "password": "MySecureP@ssw0rd123",
                "username": "admin"
            }
        })
    }

    fn example_output(&self) -> String {
        "Password strength: Strong\nPassword meets all security requirements".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemSecurity
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
        let username = parameters
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let (is_weak, reason) = is_password_weak(password);
        let strength = get_password_strength(password);
        let mut result = format!("Password strength: {:?}\n", strength);
        result.push_str(&format!(
            "Password: {}\n",
            if is_weak { "WEAK" } else { "SECURE" }
        ));
        result.push_str(&format!("Reason: {}\n", reason));
        // Additional recommendations
        if is_weak {
            result.push_str("\nRecommendations:\n");
            if password.len() < 8 {
                result.push_str("- Use at least 8 characters\n");
            }
            if !password.chars().any(|c| c.is_uppercase()) {
                result.push_str("- Include uppercase letters\n");
            }
            if !password.chars().any(|c| c.is_lowercase()) {
                result.push_str("- Include lowercase letters\n");
            }
            if !password.chars().any(|c| c.is_ascii_digit()) {
                result.push_str("- Include numbers\n");
            }
            if !password.chars().any(|c| !c.is_alphanumeric()) {
                result.push_str("- Include special characters (!@#$%^&*)\n");
            }
        }
        Ok(result)
    }
}
