//! RSA signature verification skill
//!
//! This driver provides functionality to verify digital signatures using RSA public key.
use super::common::{from_base64, rsa_verify};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for RSA signature verification
///
/// Verifies a digital signature using RSA public key.
#[derive(Debug)]
pub struct RsaVerifyDriver;
#[async_trait::async_trait]
impl Driver for RsaVerifyDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "rsa_verify"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Verify RSA digital signature using public key"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to verify an RSA signature. Provide the public key, original data, and the signature."
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
                description: "Original data that was signed".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello World".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "signature".to_string(),
                param_type: "string".to_string(),
                description: "Base64 signature to verify".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("7f83b1657ff1fc53...".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "rsa_verify",
            "parameters": {
                "public_key": "-----BEGIN PUBLIC KEY-----\n...",
                "data": "Hello World",
                "signature": "7f83b1657ff1fc53..."
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Signature is valid".to_string();
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
        debug!("Executing rsa_verify driver");
        let public_key = parameters.get("public_key").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'public_key' parameter");
            DriverError::missing_parameter("public_key")
        })?;
        let data = parameters.get("data").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'data' parameter");
            DriverError::missing_parameter("data")
        })?;
        let signature = parameters.get("signature").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'signature' parameter");
            DriverError::missing_parameter("signature")
        })?;
        debug!("Data length: {} characters", data.len());
        let signature_bytes = from_base64(signature).map_err(|e| DriverError::execution(format!("Failed to decode Base64 signature: {}", e)))?;
        debug!("Signature length: {} bytes", signature_bytes.len());
        let is_valid = rsa_verify(public_key, data.as_bytes(), &signature_bytes)
            .map_err(|e| DriverError::execution(format!("RSA verification failed: {}", e)))?;
        if is_valid {
            info!("Signature is valid");
            return Ok("Signature is valid".to_string());
        } else {
            info!("Signature is invalid");
            return Ok("Signature is invalid".to_string());
        }
    }
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating rsa_verify parameters");
        if parameters.get("public_key").is_none() {
            return Err(DriverError::missing_parameter("public_key"));
        }
        if parameters.get("data").is_none() {
            return Err(DriverError::missing_parameter("data"));
        }
        if parameters.get("signature").is_none() {
            return Err(DriverError::missing_parameter("signature"));
        }
        debug!("Parameter validation passed");
        return Ok(());
    }
}
