//! RSA decryption driver

use super::common::{from_base64, rsa_decrypt};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for RSA decryption
///
/// # Description
/// Decrypts data using RSA private key. The data is expected to be Base64 encoded.
///
/// # Parameters
/// * `private_key` (required) - RSA private key in PEM format
/// * `data` (required) - Base64 encrypted data
/// * `encoding` (optional) - Output encoding: "string" (default) or "hex"
///
/// # Example
/// ```
/// Input: private_key="-----BEGIN PRIVATE KEY-----...", data="7f83b1657ff1fc53..."
/// Output: "Decrypted: Hello World"
/// ```
#[derive(Debug)]
pub struct RsaDecryptDriver;

#[async_trait::async_trait]
impl Driver for RsaDecryptDriver {
    fn name(&self) -> &str {
        "rsa_decrypt"
    }

    fn description(&self) -> &str {
        "Decrypt data using RSA private key"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to decrypt data with an RSA private key. Provide the private key in PEM format."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "private_key".to_string(),
                param_type: "string".to_string(),
                description: "RSA private key in PEM format".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "-----BEGIN PRIVATE KEY-----\n...".to_string(),
                )),
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "rsa_decrypt",
            "parameters": {
                "private_key": "-----BEGIN PRIVATE KEY-----\n...",
                "data": "7f83b1657ff1fc53..."
            }
        })
    }

    fn example_output(&self) -> String {
        "Decrypted: Hello World".to_string()
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
        let private_key = parameters
            .get("private_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'private_key' parameter"))?;
        let data_str = parameters
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'data' parameter"))?;
        let encoding = parameters
            .get("encoding")
            .and_then(|v| v.as_str())
            .unwrap_or("string");

        let data = from_base64(data_str)?;
        let decrypted = rsa_decrypt(private_key, &data)?;

        let output = match encoding {
            "hex" => {
                use super::common::to_hex;
                to_hex(&decrypted)
            }
            _ => String::from_utf8(decrypted)
                .map_err(|e| anyhow::anyhow!("Decrypted data is not valid UTF-8: {}", e))?,
        };

        Ok(format!("Decrypted: {}", output))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("private_key")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: private_key"))?;
        parameters
            .get("data")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: data"))?;
        Ok(())
    }
}
