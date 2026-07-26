//! Base64 decoding skill
//!
//! This driver provides functionality to decode a Base64 string back to its
//! original text representation.
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for Base64 decoding
///
/// Decodes a Base64 string back to its original text representation.
/// The decoded data must be valid UTF-8.
#[derive(Debug)]
pub struct Base64DecodeDriver;
#[async_trait::async_trait]
impl Driver for Base64DecodeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "base64_decode"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Decode a Base64 string to original text"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to decode Base64 encoded text"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "input".to_string(),
            param_type: "string".to_string(),
            description: "Base64 string to decode".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("SGVsbG8gV29ybGQ=".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "base64_decode",
            "parameters": {
                "input": "SGVsbG8gV29ybGQ="
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Decoded: Hello World".to_string();
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
        debug!("Executing base64_decode driver");
        let input = parameters.get("input").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'input' parameter");
            DriverError::missing_parameter("input")
        })?;
        debug!("Input length: {} characters", input.len());
        let decoded = STANDARD.decode(input).map_err(|e| {
            warn!("Invalid Base64 string: {}", e);
            DriverError::execution(format!("Invalid Base64 string: {}", e))
        })?;
        debug!("Decoded {} bytes", decoded.len());
        let decoded_str = String::from_utf8(decoded).map_err(|e| {
            warn!("Decoded data is not valid UTF-8: {}", e);
            DriverError::execution(format!("Decoded data is not valid UTF-8: {}", e))
        })?;
        info!("Base64 decode completed successfully");
        return Ok(format!("Decoded: {}", decoded_str));
    }
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating base64_decode parameters");
        if parameters.get("input").is_none() {
            return Err(DriverError::missing_parameter("input"));
        }
        debug!("Parameter validation passed");
        return Ok(());
    }
}
