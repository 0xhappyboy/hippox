//! HMAC hash skill
//!
//! This driver provides functionality to compute HMAC using SHA256 or SHA512.
use super::common::{hmac_sha256, hmac_sha512};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for calculating HMAC
///
/// Computes HMAC (Hash-based Message Authentication Code) using SHA256 or SHA512.
#[derive(Debug)]
pub struct HashHmacDriver;
#[async_trait::async_trait]
impl Driver for HashHmacDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "hash_hmac"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Calculate HMAC (Hash-based Message Authentication Code) using SHA256 or SHA512"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to compute HMAC for message authentication. Provide the message, secret key, and optional algorithm."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "input".to_string(),
                param_type: "string".to_string(),
                description: "Message to authenticate".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello World".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Secret key for HMAC".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("secret".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "algorithm".to_string(),
                param_type: "string".to_string(),
                description: "Hash algorithm: 'sha256' or 'sha512'".to_string(),
                required: false,
                default: Some(Value::String("sha256".to_string())),
                example: Some(Value::String("sha512".to_string())),
                enum_values: Some(vec!["sha256".to_string(), "sha512".to_string()]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "hash_hmac",
            "parameters": {
                "input": "Hello World",
                "key": "secret",
                "algorithm": "sha256"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "HMAC-SHA256: 7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069".to_string();
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
        debug!("Executing hash_hmac driver");
        let input = parameters.get("input").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'input' parameter");
            DriverError::missing_parameter("input")
        })?;
        let key = parameters.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'key' parameter");
            DriverError::missing_parameter("key")
        })?;
        let algorithm = parameters.get("algorithm").and_then(|v| v.as_str()).unwrap_or("sha256");
        debug!("HMAC algorithm: {}, input length: {}, key length: {}", algorithm, input.len(), key.len());
        let result = match algorithm {
            "sha256" => hmac_sha256(key.as_bytes(), input.as_bytes()).map_err(|e| DriverError::execution(format!("HMAC-SHA256 failed: {}", e)))?,
            "sha512" => hmac_sha512(key.as_bytes(), input.as_bytes()).map_err(|e| DriverError::execution(format!("HMAC-SHA512 failed: {}", e)))?,
            _ => {
                warn!("Unsupported algorithm: {}", algorithm);
                return Err(DriverError::execution(format!("Unsupported algorithm: {}", algorithm)));
            }
        };
        info!("HMAC computed successfully");
        return Ok(format!("HMAC-{}: {}", algorithm.to_uppercase(), hex::encode(result)));
    }
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating hash_hmac parameters");
        if parameters.get("input").is_none() {
            return Err(DriverError::missing_parameter("input"));
        }
        if parameters.get("key").is_none() {
            return Err(DriverError::missing_parameter("key"));
        }
        debug!("Parameter validation passed");
        return Ok(());
    }
}
