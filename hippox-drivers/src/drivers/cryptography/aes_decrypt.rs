//! AES decryption skill

use super::common::{aes_cbc_decrypt, aes_gcm_decrypt, from_hex, to_hex};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for AES decryption
///
/// # Description
/// Decrypts data using AES symmetric encryption. Supports CBC and GCM modes.
///
/// # Parameters
/// * `key` (required) - AES key (hex string, 16/24/32 bytes for AES-128/192/256)
/// * `ciphertext` (required) - Data to decrypt (hex string)
/// * `iv_or_nonce` (required) - IV (for CBC) or nonce (for GCM) as hex string
/// * `mode` (optional) - "cbc" (default) or "gcm"
/// * `associated_data` (optional) - Additional authenticated data for GCM mode (hex string)
///
/// # Example
/// ```
/// Input: key="0123456789abcdef0123456789abcdef", ciphertext="7f83b1657ff1fc53b92dc18148a1d65d", iv_or_nonce="1234567890abcdef12345678", mode="gcm"
/// Output: "Decrypted: 48656c6c6f20576f726c64"
/// ```
#[derive(Debug)]
pub struct AesDecryptDriver;

#[async_trait::async_trait]
impl Driver for AesDecryptDriver {
    fn name(&self) -> &str {
        "aes_decrypt"
    }

    fn description(&self) -> &str {
        "Decrypt data using AES symmetric encryption (CBC or GCM mode)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to decrypt data with AES. Provide key (hex), ciphertext (hex), IV/nonce (hex), and optional mode."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "AES key as hex string (16 bytes for AES-128, 32 bytes for AES-256)"
                    .to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "0123456789abcdef0123456789abcdef".to_string(),
                )),
                enum_values: None,
            },
            DriverParameter {
                name: "ciphertext".to_string(),
                param_type: "string".to_string(),
                description: "Data to decrypt as hex string".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "7f83b1657ff1fc53b92dc18148a1d65d".to_string(),
                )),
                enum_values: None,
            },
            DriverParameter {
                name: "iv_or_nonce".to_string(),
                param_type: "string".to_string(),
                description: "IV (CBC) or nonce (GCM) as hex string".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("1234567890abcdef12345678".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "mode".to_string(),
                param_type: "string".to_string(),
                description: "Encryption mode: 'cbc' or 'gcm'".to_string(),
                required: false,
                default: Some(Value::String("cbc".to_string())),
                example: Some(Value::String("gcm".to_string())),
                enum_values: Some(vec!["cbc".to_string(), "gcm".to_string()]),
            },
            DriverParameter {
                name: "associated_data".to_string(),
                param_type: "string".to_string(),
                description: "Additional authenticated data for GCM mode (hex string)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("6164646974696f6e616c".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "aes_decrypt",
            "parameters": {
                "key": "0123456789abcdef0123456789abcdef",
                "ciphertext": "7f83b1657ff1fc53b92dc18148a1d65d",
                "iv_or_nonce": "1234567890abcdef12345678",
                "mode": "gcm"
            }
        })
    }

    fn example_output(&self) -> String {
        "Decrypted: 48656c6c6f20576f726c64".to_string()
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
        let key_hex = parameters
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'key' parameter"))?;
        let ciphertext_hex = parameters
            .get("ciphertext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'ciphertext' parameter"))?;
        let iv_or_nonce_hex = parameters
            .get("iv_or_nonce")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'iv_or_nonce' parameter"))?;
        let mode = parameters
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("cbc");

        let key = from_hex(key_hex)?;
        let ciphertext = from_hex(ciphertext_hex)?;
        let iv_or_nonce = from_hex(iv_or_nonce_hex)?;

        // Validate key length
        if key.len() != 16 && key.len() != 24 && key.len() != 32 {
            anyhow::bail!("Key must be 16 (AES-128), 24 (AES-192), or 32 (AES-256) bytes");
        }

        let plaintext = match mode {
            "cbc" => {
                if iv_or_nonce.len() != 16 {
                    anyhow::bail!("IV must be 16 bytes for CBC mode");
                }
                aes_cbc_decrypt(&key, &iv_or_nonce, &ciphertext)?
            }
            "gcm" => {
                if iv_or_nonce.len() != 12 {
                    anyhow::bail!("Nonce must be 12 bytes for GCM mode");
                }
                let associated_data = parameters
                    .get("associated_data")
                    .and_then(|v| v.as_str())
                    .map(from_hex)
                    .transpose()?;
                aes_gcm_decrypt(&key, &iv_or_nonce, &ciphertext, associated_data.as_deref())?
            }
            _ => anyhow::bail!("Unsupported mode: {}", mode),
        };

        Ok(format!("Decrypted: {}", to_hex(&plaintext)))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("key")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: key"))?;
        parameters
            .get("ciphertext")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: ciphertext"))?;
        parameters
            .get("iv_or_nonce")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: iv_or_nonce"))?;
        Ok(())
    }
}
