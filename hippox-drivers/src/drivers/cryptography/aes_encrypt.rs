//! AES encryption driver
//!
//! This driver provides functionality to encrypt data using AES symmetric encryption.
//! Supports CBC and GCM modes.
use super::common::{aes_cbc_encrypt, aes_gcm_encrypt, from_hex, to_hex};
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverError;
use crate::DriverResult;
use crate::types::{Driver, DriverParameter};
use serde_json::{Value, json};
use tracing::debug;
use tracing::info;
use tracing::warn;
use std::collections::HashMap;
/// Driver for AES encryption
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
pub struct AesEncryptDriver;
#[async_trait::async_trait]
impl Driver for AesEncryptDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "aes_encrypt"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Encrypt data using AES symmetric encryption (CBC or GCM mode)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to encrypt data with AES. Provide key (hex), plaintext (hex), and optional mode."
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
                name: "plaintext".to_string(),
                param_type: "string".to_string(),
                description: "Data to encrypt as hex string".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("48656c6c6f20576f726c64".to_string())),
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
            "action": "aes_encrypt",
            "parameters": {
                "key": "0123456789abcdef0123456789abcdef",
                "plaintext": "48656c6c6f20576f726c64",
                "mode": "gcm"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Nonce: 1234567890abcdef12345678\nCiphertext: 7f83b1657ff1fc53b92dc18148a1d65d".to_string();
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
        debug!("Executing aes_encrypt driver");
        let key_hex = parameters.get("key").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'key' parameter");
            DriverError::missing_parameter("key")
        })?;
        let plaintext_hex = parameters.get("plaintext").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'plaintext' parameter");
            DriverError::missing_parameter("plaintext")
        })?;
        let mode = parameters.get("mode").and_then(|v| v.as_str()).unwrap_or("cbc");
        debug!("Encryption mode: {}", mode);
        let key = from_hex(key_hex).map_err(|e| DriverError::execution(format!("Failed to decode key hex: {}", e)))?;
        let plaintext = from_hex(plaintext_hex).map_err(|e| DriverError::execution(format!("Failed to decode plaintext hex: {}", e)))?;
        debug!("Key length: {} bytes, plaintext length: {} bytes", key.len(), plaintext.len());
        // Validate key length
        if key.len() != 16 && key.len() != 24 && key.len() != 32 {
            warn!("Invalid key length: {} bytes", key.len());
            return Err(DriverError::validation("key", "Key must be 16 (AES-128), 24 (AES-192), or 32 (AES-256) bytes"));
        }
        let (iv_or_nonce, ciphertext) = match mode {
            "cbc" => {
                debug!("Performing AES-CBC encryption");
                let (iv, ciphertext) =
                    aes_cbc_encrypt(&key, &plaintext).map_err(|e| DriverError::execution(format!("CBC encryption failed: {}", e)))?;
                (iv, ciphertext)
            }
            "gcm" => {
                let associated_data = parameters
                    .get("associated_data")
                    .and_then(|v| v.as_str())
                    .map(from_hex)
                    .transpose()
                    .map_err(|e| DriverError::execution(format!("Failed to decode associated data: {}", e)))?;
                debug!("Performing AES-GCM encryption");
                let (nonce, ciphertext) = aes_gcm_encrypt(&key, &plaintext, associated_data.as_deref())
                    .map_err(|e| DriverError::execution(format!("GCM encryption failed: {}", e)))?;
                (nonce, ciphertext)
            }
            _ => {
                warn!("Unsupported mode: {}", mode);
                return Err(DriverError::execution(format!("Unsupported mode: {}", mode)));
            }
        };
        let mode_label = if mode == "gcm" { "Nonce" } else { "IV" };
        info!("AES encryption completed successfully");
        return Ok(format!("{}: {}\nCiphertext: {}", mode_label, to_hex(&iv_or_nonce), to_hex(&ciphertext)));
    }
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating aes_encrypt parameters");
        if parameters.get("key").is_none() {
            return Err(DriverError::missing_parameter("key"));
        }
        if parameters.get("plaintext").is_none() {
            return Err(DriverError::missing_parameter("plaintext"));
        }
        debug!("Parameter validation passed");
        return Ok(());
    }
}
