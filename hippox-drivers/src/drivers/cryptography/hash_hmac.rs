//! HMAC hash skill

use super::common::{hmac_sha256, hmac_sha512};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for calculating HMAC
///
/// # Description
/// Computes HMAC (Hash-based Message Authentication Code) using SHA256 or SHA512.
/// HMAC is used for message authentication and integrity verification.
///
/// # Parameters
/// * `input` (required) - The message to authenticate
/// * `key` (required) - The secret key
/// * `algorithm` (optional) - "sha256" (default) or "sha512"
///
/// # Example
/// ```
/// Input: input="Hello World", key="secret"
/// Output: "HMAC-SHA256: 7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069"
/// ```
#[derive(Debug)]
pub struct HashHmacDriver;

#[async_trait::async_trait]
impl Driver for HashHmacDriver {
    fn name(&self) -> &str {
        "hash_hmac"
    }

    fn description(&self) -> &str {
        "Calculate HMAC (Hash-based Message Authentication Code) using SHA256 or SHA512"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to compute HMAC for message authentication. Provide the message, secret key, and optional algorithm."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "hash_hmac",
            "parameters": {
                "input": "Hello World",
                "key": "secret",
                "algorithm": "sha256"
            }
        })
    }

    fn example_output(&self) -> String {
        "HMAC-SHA256: 7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069".to_string()
    }

    fn category(&self) -> crate::DriverCategory {
        crate::DriverCategory::Cryptography
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let input = parameters
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'input' parameter"))?;
        let key = parameters
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'key' parameter"))?;
        let algorithm = parameters
            .get("algorithm")
            .and_then(|v| v.as_str())
            .unwrap_or("sha256");

        let result = match algorithm {
            "sha256" => hmac_sha256(key.as_bytes(), input.as_bytes())?,
            "sha512" => hmac_sha512(key.as_bytes(), input.as_bytes())?,
            _ => anyhow::bail!("Unsupported algorithm: {}", algorithm),
        };

        Ok(format!(
            "HMAC-{}: {}",
            algorithm.to_uppercase(),
            hex::encode(result)
        ))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("input")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: input"))?;
        parameters
            .get("key")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: key"))?;
        Ok(())
    }
}
