//! # Random Generation Drivers Module
//!
//! This module provides various random generation capabilities including:
//! - Random numbers within a specified range
//! - Random strings with configurable character sets
//! - Random UUID v4 identifiers
//! - Secure random passwords with configurable complexity
//!
//! # Examples
//! ```
//! use std::collections::HashMap;
//! use serde_json::json;
//!
//! // Generate a random number between 1 and 100
//! let mut params = HashMap::new();
//! params.insert("min".to_string(), json!(1));
//! params.insert("max".to_string(), json!(100));
//! ```
//!
//! All skills implement the `Driver` trait and can be executed asynchronously.
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use rand::RngExt;
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// # Random Number Generation Driver
///
/// Generates a cryptographically secure random integer within a specified inclusive range.
///
/// ## Parameters
/// - `min` (optional, default: 0): The minimum value (inclusive)
/// - `max` (optional, default: 100): The maximum value (inclusive)
///
/// ## Example
/// ```json
/// {
///     "action": "random_number",
///     "parameters": {
///         "min": 1,
///         "max": 100
///     }
/// }
/// ```
///
/// ## Output
/// Returns a string in the format: `"Random number: {value}"`
#[derive(Debug)]
pub struct RandomNumberDriver;
#[async_trait::async_trait]
impl Driver for RandomNumberDriver {
    /// Returns the unique name identifier for this skill
    fn name(&self) -> &str {
        "random_number"
    }
    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Generate a random number within a specified range"
    }
    /// Returns usage guidance for AI/LLM systems
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to generate random integers"
    }
    /// Defines the parameter schema for this skill
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "min".to_string(),
                param_type: "integer".to_string(),
                description: "Minimum value (inclusive)".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(1.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "max".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum value (inclusive)".to_string(),
                required: false,
                default: Some(Value::Number(100.into())),
                example: Some(Value::Number(100.into())),
                enum_values: None,
            },
        ];
    }
    /// Provides an example JSON call format
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "random_number",
            "parameters": {
                "min": 1,
                "max": 100
            }
        }));
    }
    /// Provides an example output for documentation
    fn example_output(&self) -> String {
        return "Random number: 42".to_string();
    }
    /// Returns the skill category for organization
    fn category(&self) -> DriverCategory {
        return DriverCategory::Math;
    }
    /// Executes the random number generation logic
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing random_number driver");
        let min = parameters.get("min").and_then(|v| v.as_i64()).unwrap_or(0);
        let max = parameters.get("max").and_then(|v| v.as_i64()).unwrap_or(100);
        if min > max {
            warn!("min ({}) cannot be greater than max ({})", min, max);
            return Err(crate::DriverError::validation("min", format!("min ({}) cannot be greater than max ({})", min, max)));
        }
        debug!("Generating random number between {} and {}", min, max);
        let mut rng = rand::rng();
        let number = rng.random_range(min..=max);
        info!("Generated random number: {}", number);
        return Ok(format!("Random number: {}", number));
    }
    /// Validates the input parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        if let (Some(min), Some(max)) = (parameters.get("min"), parameters.get("max")) {
            if min.as_i64().unwrap_or(0) > max.as_i64().unwrap_or(0) {
                return Err(crate::DriverError::validation("min", "min cannot be greater than max"));
            }
        }
        return Ok(());
    }
}
/// # Random String Generation Driver
///
/// Generates a random string of specified length using configurable character sets.
///
/// ## Parameters
/// - `length` (optional, default: 10): Length of the random string (max: 1024)
/// - `charset` (optional, default: "alphanumeric"): Character set to use
///   - `alphanumeric`: Letters (both cases) + numbers
///   - `alpha`: Letters only (both cases)
///   - `numeric`: Numbers only (0-9)
///   - `hex`: Hexadecimal characters (0-9, a-f)
///
/// ## Example
/// ```json
/// {
///     "action": "random_string",
///     "parameters": {
///         "length": 16,
///         "charset": "alphanumeric"
///     }
/// }
/// ```
#[derive(Debug)]
pub struct RandomStringDriver;
#[async_trait::async_trait]
impl Driver for RandomStringDriver {
    /// Returns the unique name identifier for this skill
    fn name(&self) -> &str {
        "random_string"
    }
    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Generate a random string of specified length"
    }
    /// Returns usage guidance for AI/LLM systems
    fn usage_hint(&self) -> &str {
        "Use this skill to generate random strings for IDs, tokens, or test data"
    }
    /// Defines the parameter schema for this skill
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "length".to_string(),
                param_type: "integer".to_string(),
                description: "Length of the random string".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(16.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "charset".to_string(),
                param_type: "string".to_string(),
                description: "Character set to use (alphanumeric, alpha, numeric, hex)".to_string(),
                required: false,
                default: Some(Value::String("alphanumeric".to_string())),
                example: Some(Value::String("alphanumeric".to_string())),
                enum_values: Some(vec!["alphanumeric".to_string(), "alpha".to_string(), "numeric".to_string(), "hex".to_string()]),
            },
        ];
    }
    /// Provides an example JSON call format
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "random_string",
            "parameters": {
                "length": 16,
                "charset": "alphanumeric"
            }
        }));
    }
    /// Provides an example output for documentation
    fn example_output(&self) -> String {
        return "Random string: aB3dE9fG2hJ1kL4m".to_string();
    }
    /// Returns the skill category for organization
    fn category(&self) -> DriverCategory {
        return DriverCategory::Math;
    }
    /// Executes the random string generation logic
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing random_string driver");
        let length = parameters.get("length").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
        let charset = parameters.get("charset").and_then(|v| v.as_str()).unwrap_or("alphanumeric");
        debug!("Generating random string of length {} with charset: {}", length, charset);
        let chars: Vec<char> = match charset {
            "alpha" => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect(),
            "numeric" => "0123456789".chars().collect(),
            "hex" => "0123456789abcdef".chars().collect(),
            _ => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect(),
        };
        let mut rng = rand::rng();
        let result: String = (0..length)
            .map(|_| {
                let idx = rng.random_range(0..chars.len());
                chars[idx]
            })
            .collect();
        info!("Generated random string of length {}: {}", length, result);
        return Ok(format!("Random string: {}", result));
    }
    /// Validates the input parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        if let Some(length) = parameters.get("length").and_then(|v| v.as_u64()) {
            if length == 0 {
                return Err(crate::DriverError::validation("length", "length must be greater than 0"));
            }
            if length > 1024 {
                return Err(crate::DriverError::validation("length", "length cannot exceed 1024"));
            }
        }
        return Ok(());
    }
}
/// # Random UUID Generation Driver
///
/// Generates a random version 4 (random) UUID according to RFC 4122.
///
/// ## Parameters
/// None - this skill takes no parameters.
///
/// ## Example
/// ```json
/// {
///     "action": "random_uuid",
///     "parameters": {}
/// }
/// ```
///
/// ## Output
/// Returns a string in the format: `"UUID: {uuid}"`
#[derive(Debug)]
pub struct RandomUuidDriver;
#[async_trait::async_trait]
impl Driver for RandomUuidDriver {
    /// Returns the unique name identifier for this skill
    fn name(&self) -> &str {
        "random_uuid"
    }
    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Generate a random UUID (v4)"
    }
    /// Returns usage guidance for AI/LLM systems
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to generate a unique identifier"
    }
    /// Defines the parameter schema (no parameters for this skill)
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Provides an example JSON call format
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "random_uuid",
            "parameters": {}
        }));
    }
    /// Provides an example output for documentation
    fn example_output(&self) -> String {
        return "UUID: 550e8400-e29b-41d4-a716-446655440000".to_string();
    }
    /// Returns the skill category for organization
    fn category(&self) -> DriverCategory {
        return DriverCategory::Math;
    }
    /// Executes the random UUID generation logic
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing random_uuid driver");
        let uuid = uuid::Uuid::new_v4();
        info!("Generated UUID: {}", uuid);
        return Ok(format!("UUID: {}", uuid));
    }
}
/// # Secure Random Password Generation Driver
///
/// Generates a cryptographically secure random password with configurable character type inclusion.
/// This skill ensures passwords include at least one character from each selected type.
///
/// ## Parameters
/// - `length` (optional, default: 16): Password length (max: 128)
/// - `use_uppercase` (optional, default: true): Include uppercase letters (A-Z)
/// - `use_lowercase` (optional, default: true): Include lowercase letters (a-z)
/// - `use_numbers` (optional, default: true): Include numbers (0-9)
/// - `use_symbols` (optional, default: true): Include special symbols (!@#$%^&*()_+-=[]{}|;:,.<>?)
///
/// ## Security Note
/// The generated password uses a cryptographically secure random number generator
/// and ensures at least one character from each enabled type is included.
///
/// ## Example
/// ```json
/// {
///     "action": "random_password",
///     "parameters": {
///         "length": 20,
///         "use_uppercase": true,
///         "use_lowercase": true,
///         "use_numbers": true,
///         "use_symbols": true
///     }
/// }
/// ```
#[derive(Debug)]
pub struct RandomPasswordDriver;
#[async_trait::async_trait]
impl Driver for RandomPasswordDriver {
    /// Returns the unique name identifier for this skill
    fn name(&self) -> &str {
        "random_password"
    }
    /// Returns a human-readable description of what this skill does
    fn description(&self) -> &str {
        "Generate a secure random password with configurable complexity"
    }
    /// Returns usage guidance for AI/LLM systems
    fn usage_hint(&self) -> &str {
        "Use this skill to generate strong passwords for accounts or services"
    }
    /// Defines the parameter schema for this skill
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "length".to_string(),
                param_type: "integer".to_string(),
                description: "Password length".to_string(),
                required: false,
                default: Some(Value::Number(16.into())),
                example: Some(Value::Number(20.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "use_uppercase".to_string(),
                param_type: "boolean".to_string(),
                description: "Include uppercase letters".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "use_lowercase".to_string(),
                param_type: "boolean".to_string(),
                description: "Include lowercase letters".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "use_numbers".to_string(),
                param_type: "boolean".to_string(),
                description: "Include numbers".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "use_symbols".to_string(),
                param_type: "boolean".to_string(),
                description: "Include special symbols".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Provides an example JSON call format
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "random_password",
            "parameters": {
                "length": 16
            }
        }));
    }
    /// Provides an example output for documentation
    fn example_output(&self) -> String {
        return "Password: aB3#dE9$fG2hJ1kL".to_string();
    }
    /// Returns the skill category for organization
    fn category(&self) -> DriverCategory {
        return DriverCategory::Math;
    }
    /// Executes the secure password generation logic
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing random_password driver");
        let length = parameters.get("length").and_then(|v| v.as_u64()).unwrap_or(16) as usize;
        let use_uppercase = parameters.get("use_uppercase").and_then(|v| v.as_bool()).unwrap_or(true);
        let use_lowercase = parameters.get("use_lowercase").and_then(|v| v.as_bool()).unwrap_or(true);
        let use_numbers = parameters.get("use_numbers").and_then(|v| v.as_bool()).unwrap_or(true);
        let use_symbols = parameters.get("use_symbols").and_then(|v| v.as_bool()).unwrap_or(true);
        debug!(
            "Generating password of length {} with options: uppercase={}, lowercase={}, numbers={}, symbols={}",
            length, use_uppercase, use_lowercase, use_numbers, use_symbols
        );
        let mut char_pool = Vec::new();
        if use_uppercase {
            char_pool.extend_from_slice(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        }
        if use_lowercase {
            char_pool.extend_from_slice(b"abcdefghijklmnopqrstuvwxyz");
        }
        if use_numbers {
            char_pool.extend_from_slice(b"0123456789");
        }
        if use_symbols {
            char_pool.extend_from_slice(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
        }
        if char_pool.is_empty() {
            warn!("At least one character type must be enabled");
            return Err(crate::DriverError::validation("characters", "At least one character type must be enabled"));
        }
        let mut rng = rand::rng();
        let password: String = (0..length)
            .map(|_| {
                let idx = rng.random_range(0..char_pool.len());
                char_pool[idx] as char
            })
            .collect();
        info!("Generated password of length {}: {}", length, password);
        return Ok(format!("Password: {}", password));
    }
    /// Validates the input parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        if let Some(length) = parameters.get("length").and_then(|v| v.as_u64()) {
            if length == 0 {
                return Err(crate::DriverError::validation("length", "length must be greater than 0"));
            }
            if length > 128 {
                return Err(crate::DriverError::validation("length", "length cannot exceed 128"));
            }
        }
        return Ok(());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    /// Test RandomNumberDriver functionality
    #[tokio::test]
    async fn test_random_number_skill() {
        let skill = RandomNumberDriver;
        let params = HashMap::new();
        let result = skill.execute(&params, None, None).await.unwrap();
        assert!(result.starts_with("Random number: "));
        let num = result.trim_start_matches("Random number: ").parse::<i64>().unwrap();
        assert!(num >= 0 && num <= 100);
        let mut params = HashMap::new();
        params.insert("min".to_string(), json!(10));
        params.insert("max".to_string(), json!(20));
        let result = skill.execute(&params, None, None).await.unwrap();
        let num = result.trim_start_matches("Random number: ").parse::<i64>().unwrap();
        assert!(num >= 10 && num <= 20);
        let mut params = HashMap::new();
        params.insert("min".to_string(), json!(100));
        params.insert("max".to_string(), json!(1));
        assert!(skill.execute(&params, None, None).await.is_err());
    }
    /// Test RandomStringDriver functionality
    #[tokio::test]
    async fn test_random_string_skill() {
        let skill = RandomStringDriver;
        let params = HashMap::new();
        let result = skill.execute(&params, None, None).await.unwrap();
        let s = result.trim_start_matches("Random string: ");
        assert_eq!(s.len(), 10);
        let mut params = HashMap::new();
        params.insert("length".to_string(), json!(8));
        params.insert("charset".to_string(), json!("numeric"));
        let result = skill.execute(&params, None, None).await.unwrap();
        let s = result.trim_start_matches("Random string: ");
        assert_eq!(s.len(), 8);
        assert!(s.chars().all(|c| c.is_ascii_digit()));
        let mut params = HashMap::new();
        params.insert("length".to_string(), json!(6));
        params.insert("charset".to_string(), json!("hex"));
        let result = skill.execute(&params, None, None).await.unwrap();
        let s = result.trim_start_matches("Random string: ");
        assert_eq!(s.len(), 6);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
        let mut params = HashMap::new();
        params.insert("length".to_string(), json!(0));
        assert!(skill.validate(&params).is_err());
    }
    /// Test RandomPasswordDriver functionality
    #[tokio::test]
    async fn test_random_password_skill() {
        let skill = RandomPasswordDriver;
        let params = HashMap::new();
        let result = skill.execute(&params, None, None).await.unwrap();
        let password = result.trim_start_matches("Password: ");
        assert_eq!(password.len(), 16);
        let mut params = HashMap::new();
        params.insert("length".to_string(), json!(12));
        params.insert("use_symbols".to_string(), json!(false));
        let result = skill.execute(&params, None, None).await.unwrap();
        let password = result.trim_start_matches("Password: ");
        assert_eq!(password.len(), 12);
        assert!(password.chars().all(|c| c.is_ascii_alphanumeric()));
        let mut params = HashMap::new();
        params.insert("length".to_string(), json!(0));
        assert!(skill.validate(&params).is_err());
        let mut params = HashMap::new();
        params.insert("length".to_string(), json!(200));
        assert!(skill.validate(&params).is_err());
    }
}
