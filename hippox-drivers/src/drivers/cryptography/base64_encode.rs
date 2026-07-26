//! Base64 encoding driver
//!
//! This driver provides functionality to encode a string to Base64 format.
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for Base64 encoding
///
/// Encodes a string to Base64 format. Base64 encoding is commonly used to
/// represent binary data in an ASCII string format.
#[derive(Debug)]
pub struct Base64EncodeDriver;
#[async_trait::async_trait]
impl Driver for Base64EncodeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "base64_encode"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Encode a string to Base64"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to convert text to Base64 encoding"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "input".to_string(),
            param_type: "string".to_string(),
            description: "Input string to encode".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Hello World".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "base64_encode",
            "parameters": {
                "input": "Hello World"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Base64: SGVsbG8gV29ybGQ=".to_string();
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
        debug!("Executing base64_encode driver");
        let input = parameters.get("input").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'input' parameter");
            DriverError::missing_parameter("input")
        })?;
        debug!("Input length: {} characters", input.len());
        let encoded = STANDARD.encode(input.as_bytes());
        debug!("Encoded length: {} characters", encoded.len());
        info!("Base64 encode completed successfully");
        return Ok(format!("Base64: {}", encoded));
    }
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating base64_encode parameters");
        if parameters.get("input").is_none() {
            return Err(DriverError::missing_parameter("input"));
        }
        debug!("Parameter validation passed");
        return Ok(());
    }
}
