//! RSA encryption skill
//!
//! This driver provides functionality to encrypt data using RSA public key.
use super::common::{rsa_encrypt, to_base64};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for RSA encryption
///
/// Encrypts data using RSA public key encryption.
#[derive(Debug)]
pub struct RsaEncryptDriver;
#[async_trait::async_trait]
impl Driver for RsaEncryptDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "rsa_encrypt"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Encrypt data using RSA public key"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to encrypt data with an RSA public key. Provide the public key in PEM format."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "public_key".to_string(),
                param_type: "string".to_string(),
                description: "RSA public key in PEM format".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("-----BEGIN PUBLIC KEY-----\n...".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "data".to_string(),
                param_type: "string".to_string(),
                description: "Data to encrypt (plain text)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello World".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "encoding".to_string(),
                param_type: "string".to_string(),
                description: "Input encoding: 'string' or 'hex'".to_string(),
                required: false,
                default: Some(Value::String("string".to_string())),
                example: Some(Value::String("hex".to_string())),
                enum_values: Some(vec!["string".to_string(), "hex".to_string()]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "rsa_encrypt",
            "parameters": {
                "public_key": "-----BEGIN PUBLIC KEY-----\n...",
                "data": "Hello World"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Encrypted: 7f83b1657ff1fc53b92dc18148a1d65d...".to_string();
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
        debug!("Executing rsa_encrypt driver");
        let public_key = parameters.get("public_key").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'public_key' parameter");
            DriverError::missing_parameter("public_key")
        })?;
        let data_str = parameters.get("data").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'data' parameter");
            DriverError::missing_parameter("data")
        })?;
        let encoding = parameters.get("encoding").and_then(|v| v.as_str()).unwrap_or("string");
        debug!("RSA encryption parameters: encoding={}", encoding);
        let data = match encoding {
            "hex" => {
                use super::common::from_hex;
                from_hex(data_str).map_err(|e| DriverError::execution(format!("Failed to decode hex data: {}", e)))?
            }
            _ => data_str.as_bytes().to_vec(),
        };
        debug!("Data length: {} bytes", data.len());
        let encrypted = rsa_encrypt(public_key, &data).map_err(|e| DriverError::execution(format!("RSA encryption failed: {}", e)))?;
        debug!("Encrypted length: {} bytes", encrypted.len());
        info!("RSA encryption completed successfully");
        return Ok(format!("Encrypted: {}", to_base64(&encrypted)));
    }
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating rsa_encrypt parameters");
        if parameters.get("public_key").is_none() {
            return Err(DriverError::missing_parameter("public_key"));
        }
        if parameters.get("data").is_none() {
            return Err(DriverError::missing_parameter("data"));
        }
        debug!("Parameter validation passed");
        return Ok(());
    }
}
