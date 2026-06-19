//! RSA encryption skill

use super::common::{rsa_encrypt, to_base64};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for RSA encryption
///
/// # Description
/// Encrypts data using RSA public key encryption. The data is encrypted using the provided
/// public key (PEM format). The result is Base64 encoded.
///
/// # Parameters
/// * `public_key` (required) - RSA public key in PEM format
/// * `data` (required) - Data to encrypt (string or hex)
/// * `encoding` (optional) - "string" (default) or "hex"
///
/// # Example
/// ```
/// Input: public_key="-----BEGIN PUBLIC KEY-----...", data="Hello World"
/// Output: "Encrypted: 7f83b1657ff1fc53b92dc18148a1d65d..."
/// ```
#[derive(Debug)]
pub struct RsaEncryptDriver;

#[async_trait::async_trait]
impl Driver for RsaEncryptDriver {
    fn name(&self) -> &str {
        "rsa_encrypt"
    }

    fn description(&self) -> &str {
        "Encrypt data using RSA public key"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to encrypt data with an RSA public key. Provide the public key in PEM format."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "rsa_encrypt",
            "parameters": {
                "public_key": "-----BEGIN PUBLIC KEY-----\n...",
                "data": "Hello World"
            }
        })
    }

    fn example_output(&self) -> String {
        "Encrypted: 7f83b1657ff1fc53b92dc18148a1d65d...".to_string()
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
        let public_key = parameters
            .get("public_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'public_key' parameter"))?;
        let data_str = parameters
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'data' parameter"))?;
        let encoding = parameters
            .get("encoding")
            .and_then(|v| v.as_str())
            .unwrap_or("string");

        let data = match encoding {
            "hex" => {
                use super::common::from_hex;
                from_hex(data_str)?
            }
            _ => data_str.as_bytes().to_vec(),
        };

        let encrypted = rsa_encrypt(public_key, &data)?;
        Ok(format!("Encrypted: {}", to_base64(&encrypted)))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("public_key")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: public_key"))?;
        parameters
            .get("data")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: data"))?;
        Ok(())
    }
}
