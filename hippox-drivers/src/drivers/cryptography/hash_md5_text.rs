//! MD5 hash skill for text
//!
//! This driver provides functionality to calculate MD5 hash of a text string.
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for calculating MD5 hash of a string
///
/// Computes the MD5 hash (128-bit) of a given input string and returns it as a hexadecimal string.
#[derive(Debug)]
pub struct HashMd5TextDriver;
#[async_trait::async_trait]
impl Driver for HashMd5TextDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "hash_md5_text"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Calculate MD5 hash of a text string"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need to compute MD5 hash for a text string. For file hashing, use file/hash_md5."
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
            "action": "hash_md5_text",
            "parameters": {
                "input": "Hello World"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "MD5: b10a8db164e0754105b7a99be72e3fe5".to_string();
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
        debug!("Executing hash_md5_text driver");
        let input = parameters.get("input").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'input' parameter");
            DriverError::missing_parameter("input")
        })?;
        debug!("Input length: {} characters", input.len());
        let digest = md5::compute(input.as_bytes());
        info!("MD5 hash computed successfully");
        return Ok(format!("MD5: {:x}", digest));
    }
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating hash_md5_text parameters");
        if parameters.get("input").is_none() {
            return Err(DriverError::missing_parameter("input"));
        }
        debug!("Parameter validation passed");
        return Ok(());
    }
}
