//! AES encryption skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{aes_cbc_encrypt, aes_gcm_encrypt, from_hex, to_hex};
use crate::types::{Skill, SkillParameter};

/// Skill for AES encryption
///
/// # Description
/// Encrypts data using AES symmetric encryption. Supports CBC and GCM modes.
/// For GCM mode, returns both ciphertext and nonce.
///
/// # Parameters
/// * `key` (required) - AES key (hex string, 16/24/32 bytes for AES-128/192/256)
/// * `plaintext` (required) - Data to encrypt (hex string)
/// * `mode` (optional) - "cbc" (default) or "gcm"
/// * `associated_data` (optional) - Additional authenticated data for GCM mode (hex string)
///
/// # Example
/// ```
/// Input: key="0123456789abcdef0123456789abcdef", plaintext="48656c6c6f20576f726c64", mode="gcm"
/// Output: "Nonce: 1234567890abcdef12345678\nCiphertext: 7f83b1657ff1fc53b92dc18148a1d65d"
/// ```
#[derive(Debug)]
pub struct AesEncryptSkill;

#[async_trait::async_trait]
impl Skill for AesEncryptSkill {
    fn name(&self) -> &str {
        "aes_encrypt"
    }

    fn description(&self) -> &str {
        "Encrypt data using AES symmetric encryption (CBC or GCM mode)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to encrypt data with AES. Provide key (hex), plaintext (hex), and optional mode."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
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
            SkillParameter {
                name: "plaintext".to_string(),
                param_type: "string".to_string(),
                description: "Data to encrypt as hex string".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("48656c6c6f20576f726c64".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "mode".to_string(),
                param_type: "string".to_string(),
                description: "Encryption mode: 'cbc' or 'gcm'".to_string(),
                required: false,
                default: Some(Value::String("cbc".to_string())),
                example: Some(Value::String("gcm".to_string())),
                enum_values: Some(vec!["cbc".to_string(), "gcm".to_string()]),
            },
            SkillParameter {
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
            "action": "aes_encrypt",
            "parameters": {
                "key": "0123456789abcdef0123456789abcdef",
                "plaintext": "48656c6c6f20576f726c64",
                "mode": "gcm"
            }
        })
    }

    fn example_output(&self) -> String {
        "Nonce: 1234567890abcdef12345678\nCiphertext: 7f83b1657ff1fc53b92dc18148a1d65d".to_string()
    }

    fn category(&self) -> crate::SkillCategory {
        crate::SkillCategory::Cryptography
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let key_hex = parameters
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'key' parameter"))?;
        let plaintext_hex = parameters
            .get("plaintext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'plaintext' parameter"))?;
        let mode = parameters
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("cbc");

        let key = from_hex(key_hex)?;
        let plaintext = from_hex(plaintext_hex)?;

        // Validate key length
        if key.len() != 16 && key.len() != 24 && key.len() != 32 {
            anyhow::bail!("Key must be 16 (AES-128), 24 (AES-192), or 32 (AES-256) bytes");
        }

        let (iv_or_nonce, ciphertext) = match mode {
            "cbc" => {
                let (iv, ciphertext) = aes_cbc_encrypt(&key, &plaintext)?;
                (iv, ciphertext)
            }
            "gcm" => {
                let associated_data = parameters
                    .get("associated_data")
                    .and_then(|v| v.as_str())
                    .map(from_hex)
                    .transpose()?;
                let (nonce, ciphertext) =
                    aes_gcm_encrypt(&key, &plaintext, associated_data.as_deref())?;
                (nonce, ciphertext)
            }
            _ => anyhow::bail!("Unsupported mode: {}", mode),
        };

        let mode_label = if mode == "gcm" { "Nonce" } else { "IV" };
        Ok(format!(
            "{}: {}\nCiphertext: {}",
            mode_label,
            to_hex(&iv_or_nonce),
            to_hex(&ciphertext)
        ))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("key")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: key"))?;
        parameters
            .get("plaintext")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: plaintext"))?;
        Ok(())
    }
}
