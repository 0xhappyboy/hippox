//! Password verification skill
//!
//! This driver provides functionality to verify a password against a stored hash.
use super::common::{argon2_verify, bcrypt_verify};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for verifying passwords
///
/// Verifies a password against a stored hash.
#[derive(Debug)]
pub struct PasswordVerifyDriver;
#[async_trait::async_trait]
impl Driver for PasswordVerifyDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "password_verify"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Verify a password against a stored hash"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to verify passwords during authentication. Supports bcrypt and Argon2id hashes."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Password to verify".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("MySecureP@ssw0rd".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "hash".to_string(),
                param_type: "string".to_string(),
                description: "Stored password hash".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("$2b$12$abc123def456...".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "password_verify",
            "parameters": {
                "password": "MySecureP@ssw0rd",
                "hash": "$2b$12$abc123def456..."
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Password matches".to_string();
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
        debug!("Executing password_verify driver");
        let password = parameters.get("password").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'password' parameter");
            DriverError::missing_parameter("password")
        })?;
        let hash = parameters.get("hash").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'hash' parameter");
            DriverError::missing_parameter("hash")
        })?;
        debug!("Verifying password against hash");
        // Detect algorithm from hash format
        let is_valid = if hash.starts_with("$2") {
            debug!("Detected bcrypt hash format");
            bcrypt_verify(password, hash).map_err(|e| DriverError::execution(format!("bcrypt verification failed: {}", e)))?
        } else if hash.starts_with("$argon2id") {
            debug!("Detected Argon2id hash format");
            argon2_verify(password, hash).map_err(|e| DriverError::execution(format!("Argon2id verification failed: {}", e)))?
        } else {
            warn!("Unsupported hash format: {}", hash);
            return Err(DriverError::execution(format!("Unsupported hash format: {}", hash)));
        };
        if is_valid {
            info!("Password matches");
            return Ok("Password matches".to_string());
        } else {
            info!("Password does not match");
            return Ok("Password does not match".to_string());
        }
    }
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating password_verify parameters");
        if parameters.get("password").is_none() {
            return Err(DriverError::missing_parameter("password"));
        }
        if parameters.get("hash").is_none() {
            return Err(DriverError::missing_parameter("hash"));
        }
        debug!("Parameter validation passed");
        return Ok(());
    }
}
