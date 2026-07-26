//! SHA512 hash skill for text
//!
//! This driver provides functionality to calculate SHA512 hash of a text string.
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use sha2::{Digest, Sha512};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for calculating SHA512 hash of a string
///
/// Computes the SHA512 hash (512-bit) of a given input string and returns it as a hexadecimal string.
#[derive(Debug)]
pub struct HashSha512TextDriver;
#[async_trait::async_trait]
impl Driver for HashSha512TextDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "hash_sha512_text"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Calculate SHA512 hash of a text string"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to compute SHA512 hash for a text string. For file hashing, use file/hash_sha512."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "input".to_string(),
            param_type: "string".to_string(),
            description: "Input string to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Hello World".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "hash_sha512_text",
            "parameters": {
                "input": "Hello World"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "SHA512: 2c74fd17edafd80e8447b0d46741ee243b7eb74dd2149a0ab1b9246fb30382f27e853d8585719e0e67cbda0daa8f51671064615d645ae27acb15bfb1447f459b".to_string();
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
        debug!("Executing hash_sha512_text driver");
        let input = parameters.get("input").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'input' parameter");
            DriverError::missing_parameter("input")
        })?;
        debug!("Input length: {} characters", input.len());
        let mut hasher = Sha512::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        info!("SHA512 hash computed successfully");
        return Ok(format!("SHA512: {}", hex::encode(result)));
    }
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating hash_sha512_text parameters");
        if parameters.get("input").is_none() {
            return Err(DriverError::missing_parameter("input"));
        }
        debug!("Parameter validation passed");
        return Ok(());
    }
}
