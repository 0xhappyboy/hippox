//! Password hashing skill
//!
//! This driver provides functionality to hash passwords using bcrypt or Argon2id.
use super::common::{argon2_hash, bcrypt_hash, validate_password_strength};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for hashing passwords
///
/// Hashes passwords using bcrypt or Argon2id for secure storage.
#[derive(Debug)]
pub struct PasswordHashDriver;
#[async_trait::async_trait]
impl Driver for PasswordHashDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "password_hash"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Hash a password using bcrypt or Argon2id"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to securely hash passwords for storage. Supports bcrypt (default) and Argon2id."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Password to hash".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("MySecureP@ssw0rd".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "algorithm".to_string(),
                param_type: "string".to_string(),
                description: "Hash algorithm: 'bcrypt' or 'argon2id'".to_string(),
                required: false,
                default: Some(Value::String("bcrypt".to_string())),
                example: Some(Value::String("argon2id".to_string())),
                enum_values: Some(vec!["bcrypt".to_string(), "argon2id".to_string()]),
            },
            DriverParameter {
                name: "cost".to_string(),
                param_type: "integer".to_string(),
                description: "Cost factor for bcrypt (4-31, default 12)".to_string(),
                required: false,
                default: Some(Value::Number(12.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "validate".to_string(),
                param_type: "boolean".to_string(),
                description: "Validate password strength (default true)".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "password_hash",
            "parameters": {
                "password": "MySecureP@ssw0rd",
                "algorithm": "bcrypt",
                "cost": 12
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Hashed: $2b$12$abc123def456...".to_string();
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
        debug!("Executing password_hash driver");
        let password = parameters.get("password").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'password' parameter");
            DriverError::missing_parameter("password")
        })?;
        let algorithm = parameters.get("algorithm").and_then(|v| v.as_str()).unwrap_or("bcrypt");
        let cost = parameters.get("cost").and_then(|v| v.as_u64()).unwrap_or(12) as u32;
        let validate = parameters.get("validate").and_then(|v| v.as_bool()).unwrap_or(true);
        debug!("Password hashing parameters: algorithm={}, cost={}, validate={}", algorithm, cost, validate);
        if validate {
            debug!("Validating password strength");
            validate_password_strength(password).map_err(|e| DriverError::validation("password", format!("Password validation failed: {}", e)))?;
            debug!("Password strength validation passed");
        }
        let hashed = match algorithm {
            "bcrypt" => {
                if cost < 4 || cost > 31 {
                    warn!("Invalid cost value: {}", cost);
                    return Err(DriverError::validation("cost", "Cost must be between 4 and 31"));
                }
                debug!("Performing bcrypt hash with cost {}", cost);
                bcrypt_hash(password, cost).map_err(|e| DriverError::execution(format!("bcrypt hashing failed: {}", e)))?
            }
            "argon2id" => {
                debug!("Performing Argon2id hash");
                argon2_hash(password).map_err(|e| DriverError::execution(format!("Argon2id hashing failed: {}", e)))?
            }
            _ => {
                warn!("Unsupported algorithm: {}", algorithm);
                return Err(DriverError::execution(format!("Unsupported algorithm: {}", algorithm)));
            }
        };
        info!("Password hashed successfully");
        return Ok(format!("Hashed: {}", hashed));
    }
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating password_hash parameters");
        if parameters.get("password").is_none() {
            return Err(DriverError::missing_parameter("password"));
        }
        debug!("Parameter validation passed");
        return Ok(());
    }
}
