//! RSA decryption driver
//!
//! This driver provides functionality to decrypt data using RSA private key.
use super::common::{from_base64, rsa_decrypt};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for RSA decryption
///
/// Decrypts data using RSA private key. The data is expected to be Base64 encoded.
#[derive(Debug)]
pub struct RsaDecryptDriver;
#[async_trait::async_trait]
impl Driver for RsaDecryptDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "rsa_decrypt"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Decrypt data using RSA private key"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to decrypt data with an RSA private key. Provide the private key in PEM format."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "private_key".to_string(),
                param_type: "string".to_string(),
                description: "RSA private key in PEM format".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("-----BEGIN PRIVATE KEY-----\n...".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "data".to_string(),
                param_type: "string".to_string(),
                description: "Base64 encrypted data".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("7f83b1657ff1fc53...".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "encoding".to_string(),
                param_type: "string".to_string(),
                description: "Output encoding: 'string' or 'hex'".to_string(),
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
            "action": "rsa_decrypt",
            "parameters": {
                "private_key": "-----BEGIN PRIVATE KEY-----\n...",
                "data": "7f83b1657ff1fc53..."
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Decrypted: Hello World".to_string();
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
        debug!("Executing rsa_decrypt driver");
        let private_key = parameters.get("private_key").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'private_key' parameter");
            DriverError::missing_parameter("private_key")
        })?;
        let data_str = parameters.get("data").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'data' parameter");
            DriverError::missing_parameter("data")
        })?;
        let encoding = parameters.get("encoding").and_then(|v| v.as_str()).unwrap_or("string");
        debug!("RSA decryption parameters: encoding={}", encoding);
        let data = from_base64(data_str).map_err(|e| DriverError::execution(format!("Failed to decode Base64 data: {}", e)))?;
        debug!("Data length: {} bytes", data.len());
        let decrypted = rsa_decrypt(private_key, &data).map_err(|e| DriverError::execution(format!("RSA decryption failed: {}", e)))?;
        debug!("Decrypted {} bytes", decrypted.len());
        let output = match encoding {
            "hex" => {
                use super::common::to_hex;
                to_hex(&decrypted)
            }
            _ => String::from_utf8(decrypted).map_err(|e| DriverError::execution(format!("Decrypted data is not valid UTF-8: {}", e)))?,
        };
        info!("RSA decryption completed successfully");
        return Ok(format!("Decrypted: {}", output));
    }
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating rsa_decrypt parameters");
        if parameters.get("private_key").is_none() {
            return Err(DriverError::missing_parameter("private_key"));
        }
        if parameters.get("data").is_none() {
            return Err(DriverError::missing_parameter("data"));
        }
        debug!("Parameter validation passed");
        return Ok(());
    }
}
