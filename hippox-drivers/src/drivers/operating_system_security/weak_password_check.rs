//! Weak password detection driver
//!
//! This driver provides functionality to check if a password is weak or meets security requirements.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_security::common::{get_password_strength, is_password_weak},
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for checking weak passwords
#[derive(Debug)]
pub struct WeakPasswordCheckDriver;
#[async_trait::async_trait]
impl Driver for WeakPasswordCheckDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_weak_password_check"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Check if a password is weak or meets security requirements"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to test password strength. Checks against common weak passwords, length, complexity, and patterns."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Password to check".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("MySecureP@ssw0rd123".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "Associated username (optional, for context)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("admin".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "security_weak_password_check",
            "parameters": {
                "password": "MySecureP@ssw0rd123",
                "username": "admin"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Password strength: Strong\nPassword meets all security requirements".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemSecurity;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing security_weak_password_check driver");
        let password = parameters.get("password").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("password"))?;
        let _username = parameters.get("username").and_then(|v| v.as_str()).unwrap_or("");
        info!("Checking password strength");
        let (is_weak, reason) = is_password_weak(password);
        let strength = get_password_strength(password);
        let mut result = format!("Password strength: {:?}\n", strength);
        result.push_str(&format!("Password: {}\n", if is_weak { "WEAK" } else { "SECURE" }));
        result.push_str(&format!("Reason: {}\n", reason));
        // Additional recommendations
        if is_weak {
            info!("Password is weak: {}", reason);
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
        } else {
            info!("Password is secure");
        }
        return Ok(result);
    }
}
