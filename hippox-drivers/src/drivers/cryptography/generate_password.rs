//! Password generation driver
//!
//! This driver provides functionality to generate secure random passwords.
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use super::common::{generate_random_bytes, validate_password_strength};
/// Driver for generating secure passwords
#[derive(Debug)]
pub struct GeneratePasswordDriver;
#[async_trait::async_trait]
impl Driver for GeneratePasswordDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "generate_password"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Generate secure random passwords"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to generate secure passwords for users or services."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "length".to_string(),
                param_type: "integer".to_string(),
                description: "Password length (default: 16)".to_string(),
                required: false,
                default: Some(Value::Number(16.into())),
                example: Some(Value::Number(20.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "count".to_string(),
                param_type: "integer".to_string(),
                description: "Number of passwords to generate (default: 1)".to_string(),
                required: false,
                default: Some(Value::Number(1.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "special_chars".to_string(),
                param_type: "string".to_string(),
                description: "Custom special characters (default: !@#$%^&*()_+-=)".to_string(),
                required: false,
                default: Some(Value::String("!@#$%^&*()_+-=".to_string())),
                example: Some(Value::String("!@#$%".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "generate_password",
            "parameters": {
                "length": 16,
                "count": 1
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Generated password: Kx9#mP2$vL5@nQ8!rT3".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> crate::DriverCategory {
        return crate::DriverCategory::Cryptography;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing generate_password driver");
        let length = parameters.get("length").and_then(|v| v.as_u64()).unwrap_or(16) as usize;
        let count = parameters.get("count").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        let custom_special = parameters.get("special_chars").and_then(|v| v.as_str()).unwrap_or("!@#$%^&*()_+-=");
        debug!("Password generation parameters: length={}, count={}, special_chars={}", length, count, custom_special);
        if length < 8 {
            warn!("Password length too short: {}", length);
            return Err(DriverError::validation("length", "Password length must be at least 8 characters"));
        }
        if count == 0 {
            warn!("Count is zero");
            return Err(DriverError::validation("count", "Count must be greater than 0"));
        }
        let mut passwords = Vec::new();
        for i in 0..count {
            debug!("Generating password {}/{}", i + 1, count);
            let password = generate_secure_password(length, custom_special)
                .map_err(|e| DriverError::execution(format!("Failed to generate password: {}", e)))?;
            passwords.push(password);
        }
        info!("Generated {} password(s)", passwords.len());
        if passwords.len() == 1 {
            return Ok(format!("Generated password: {}", passwords[0]));
        } else {
            let mut output = String::from("Generated passwords:\n");
            for (i, pwd) in passwords.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", i + 1, pwd));
            }
            return Ok(output);
        }
    }
    /// Validate parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating generate_password parameters");
        debug!("Parameter validation passed");
        return Ok(());
    }
}
/// Generate a secure password with required character types
fn generate_secure_password(length: usize, special_chars: &str) -> DriverResult<String> {
    let uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let lowercase = "abcdefghijklmnopqrstuvwxyz";
    let digits = "0123456789";
    let all_chars = format!("{}{}{}{}", uppercase, lowercase, digits, special_chars);
    let all_chars_bytes = all_chars.as_bytes();
    let all_chars_len = all_chars_bytes.len();
    let random_bytes = generate_random_bytes(length).map_err(|e| DriverError::execution(format!("Failed to generate random bytes: {}", e)))?;
    let mut password_chars: Vec<char> = Vec::with_capacity(length);
    for b in random_bytes {
        let idx = (b as usize) % all_chars_len;
        password_chars.push(all_chars_bytes[idx] as char);
    }
    let mut chars: Vec<char> = password_chars.into_iter().collect();
    // Ensure at least one uppercase letter
    if !chars.iter().any(|c| c.is_uppercase()) {
        let idx = rng_index(length);
        chars[idx] = uppercase.chars().nth(rng_index(uppercase.len())).unwrap();
    }
    // Ensure at least one lowercase letter
    if !chars.iter().any(|c| c.is_lowercase()) {
        let idx = rng_index(length);
        chars[idx] = lowercase.chars().nth(rng_index(lowercase.len())).unwrap();
    }
    // Ensure at least one digit
    if !chars.iter().any(|c| c.is_ascii_digit()) {
        let idx = rng_index(length);
        chars[idx] = digits.chars().nth(rng_index(digits.len())).unwrap();
    }
    // Ensure at least one special character
    if !chars.iter().any(|c| special_chars.contains(*c)) {
        let idx = rng_index(length);
        chars[idx] = special_chars.chars().nth(rng_index(special_chars.len())).unwrap();
    }
    let result: String = chars.into_iter().collect();
    // Validate password strength
    validate_password_strength(&result).map_err(|e| DriverError::validation("password", format!("Generated password failed validation: {}", e)))?;
    return Ok(result);
}
fn rng_index(max: usize) -> usize {
    if max == 0 {
        return 0;
    }
    let mut bytes = [0u8; 1];
    let _ = getrandom::fill(&mut bytes);
    return (bytes[0] as usize) % max;
}
