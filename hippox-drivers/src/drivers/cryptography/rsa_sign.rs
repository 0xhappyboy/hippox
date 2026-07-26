//! RSA signature Driver
//!
//! This driver provides functionality to create digital signatures using RSA private key.
use super::common::{rsa_sign, to_base64};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for RSA signing
///
/// Creates a digital signature using RSA private key.
#[derive(Debug)]
pub struct RsaSignDriver;
#[async_trait::async_trait]
impl Driver for RsaSignDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "rsa_sign"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Create RSA digital signature using private key"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to sign data with an RSA private key. Provide the private key in PEM format."
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
                description: "Data to sign".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Hello World".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "rsa_sign",
            "parameters": {
                "private_key": "-----BEGIN PRIVATE KEY-----\n...",
                "data": "Hello World"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Signature: 7f83b1657ff1fc53b92dc18148a1d65d...".to_string();
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
        debug!("Executing rsa_sign driver");
        let private_key = parameters.get("private_key").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'private_key' parameter");
            DriverError::missing_parameter("private_key")
        })?;
        let data = parameters.get("data").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'data' parameter");
            DriverError::missing_parameter("data")
        })?;
        debug!("Data length: {} characters", data.len());
        let signature = rsa_sign(private_key, data.as_bytes()).map_err(|e| DriverError::execution(format!("RSA signing failed: {}", e)))?;
        debug!("Signature length: {} bytes", signature.len());
        info!("RSA signature created successfully");
        return Ok(format!("Signature: {}", to_base64(&signature)));
    }
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating rsa_sign parameters");
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
