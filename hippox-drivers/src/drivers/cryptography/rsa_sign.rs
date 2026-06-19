//! RSA signature Driver

use super::common::{rsa_sign, to_base64};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for RSA signing
///
/// # Description
/// Creates a digital signature using RSA private key. The signature is Base64 encoded.
///
/// # Parameters
/// * `private_key` (required) - RSA private key in PEM format
/// * `data` (required) - Data to sign (string)
///
/// # Example
/// ```
/// Input: private_key="-----BEGIN PRIVATE KEY-----...", data="Hello World"
/// Output: "Signature: 7f83b1657ff1fc53b92dc18148a1d65d..."
/// ```
#[derive(Debug)]
pub struct RsaSignDriver;

#[async_trait::async_trait]
impl Driver for RsaSignDriver {
    fn name(&self) -> &str {
        "rsa_sign"
    }

    fn description(&self) -> &str {
        "Create RSA digital signature using private key"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to sign data with an RSA private key. Provide the private key in PEM format."
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
                description: "Data to sign".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello World".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "rsa_sign",
            "parameters": {
                "private_key": "-----BEGIN PRIVATE KEY-----\n...",
                "data": "Hello World"
            }
        })
    }

    fn example_output(&self) -> String {
        "Signature: 7f83b1657ff1fc53b92dc18148a1d65d...".to_string()
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
        let data = parameters
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'data' parameter"))?;

        let signature = rsa_sign(private_key, data.as_bytes())?;
        Ok(format!("Signature: {}", to_base64(&signature)))
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
