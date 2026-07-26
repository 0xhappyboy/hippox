//! AES decryption skill
//!
//! This driver provides functionality to decrypt data using AES symmetric encryption.
//! Supports CBC and GCM modes.
use super::common::{aes_cbc_decrypt, aes_gcm_decrypt, from_hex, to_hex};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for AES decryption
///
/// Decrypts data using AES symmetric encryption. Supports CBC and GCM modes.
#[derive(Debug)]
pub struct AesDecryptDriver;
#[async_trait::async_trait]
impl Driver for AesDecryptDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "aes_decrypt"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Decrypt data using AES symmetric encryption (CBC or GCM mode)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to decrypt data with AES. Provide key (hex), ciphertext (hex), IV/nonce (hex), and optional mode."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "AES key as hex string (16 bytes for AES-128, 32 bytes for AES-256)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("0123456789abcdef0123456789abcdef".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "ciphertext".to_string(),
                param_type: "string".to_string(),
                description: "Data to decrypt as hex string".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("7f83b1657ff1fc53b92dc18148a1d65d".to_string())),
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
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "aes_decrypt",
            "parameters": {
                "key": "0123456789abcdef0123456789abcdef",
                "ciphertext": "7f83b1657ff1fc53b92dc18148a1d65d",
                "iv_or_nonce": "1234567890abcdef12345678",
                "mode": "gcm"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Decrypted: 48656c6c6f20576f726c64".to_string();
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
        debug!("Executing aes_decrypt driver");
        let key_hex = parameters.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'key' parameter");
            DriverError::missing_parameter("key")
        })?;
        let ciphertext_hex = parameters.get("ciphertext").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'ciphertext' parameter");
            DriverError::missing_parameter("ciphertext")
        })?;
        let iv_or_nonce_hex = parameters.get("iv_or_nonce").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'iv_or_nonce' parameter");
            DriverError::missing_parameter("iv_or_nonce")
        })?;
        let mode = parameters.get("mode").and_then(|v| v.as_str()).unwrap_or("cbc");
        debug!("Decryption mode: {}", mode);
        let key = from_hex(key_hex).map_err(|e| DriverError::execution(format!("Failed to decode key hex: {}", e)))?;
        let ciphertext = from_hex(ciphertext_hex).map_err(|e| DriverError::execution(format!("Failed to decode ciphertext hex: {}", e)))?;
        let iv_or_nonce = from_hex(iv_or_nonce_hex).map_err(|e| DriverError::execution(format!("Failed to decode IV/nonce hex: {}", e)))?;
        debug!("Key length: {} bytes", key.len());
        // Validate key length
        if key.len() != 16 && key.len() != 24 && key.len() != 32 {
            warn!("Invalid key length: {} bytes", key.len());
            return Err(DriverError::validation("key", "Key must be 16 (AES-128), 24 (AES-192), or 32 (AES-256) bytes"));
        }
        let plaintext = match mode {
            "cbc" => {
                if iv_or_nonce.len() != 16 {
                    warn!("Invalid IV length: {} bytes", iv_or_nonce.len());
                    return Err(DriverError::validation("iv_or_nonce", "IV must be 16 bytes for CBC mode"));
                }
                debug!("Performing AES-CBC decryption");
                aes_cbc_decrypt(&key, &iv_or_nonce, &ciphertext).map_err(|e| DriverError::execution(format!("CBC decryption failed: {}", e)))?
            }
            "gcm" => {
                if iv_or_nonce.len() != 12 {
                    warn!("Invalid nonce length: {} bytes", iv_or_nonce.len());
                    return Err(DriverError::validation("iv_or_nonce", "Nonce must be 12 bytes for GCM mode"));
                }
                let associated_data = parameters
                    .get("associated_data")
                    .and_then(|v| v.as_str())
                    .map(from_hex)
                    .transpose()
                    .map_err(|e| DriverError::execution(format!("Failed to decode associated data: {}", e)))?;
                debug!("Performing AES-GCM decryption");
                aes_gcm_decrypt(&key, &iv_or_nonce, &ciphertext, associated_data.as_deref())
                    .map_err(|e| DriverError::execution(format!("GCM decryption failed: {}", e)))?
            }
            _ => {
                warn!("Unsupported mode: {}", mode);
                return Err(DriverError::execution(format!("Unsupported mode: {}", mode)));
            }
        };
        info!("AES decryption completed successfully");
        return Ok(format!("Decrypted: {}", to_hex(&plaintext)));
    }
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating aes_decrypt parameters");
        if parameters.get("key").is_none() {
            return Err(DriverError::missing_parameter("key"));
        }
        if parameters.get("ciphertext").is_none() {
            return Err(DriverError::missing_parameter("ciphertext"));
        }
        if parameters.get("iv_or_nonce").is_none() {
            return Err(DriverError::missing_parameter("iv_or_nonce"));
        }
        debug!("Parameter validation passed");
        return Ok(());
    }
}
