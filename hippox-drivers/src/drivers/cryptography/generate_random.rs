//! Random generation driver
//!
//! This driver provides functionality to generate cryptographically secure random data.
use super::common::{generate_random_bytes, generate_random_hex, generate_random_string};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for generating random data
///
/// Generates cryptographically secure random data in various formats: bytes, hex, or string.
#[derive(Debug)]
pub struct GenerateRandomDriver;
#[async_trait::async_trait]
impl Driver for GenerateRandomDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "generate_random"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Generate cryptographically secure random data (bytes, hex, or string)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need random data for keys, tokens, or testing purposes."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "length".to_string(),
                param_type: "integer".to_string(),
                description: "Length of random data to generate".to_string(),
                required: true,
                default: None,
                example: Some(Value::Number(16.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "format".to_string(),
                param_type: "string".to_string(),
                description: "Output format: 'bytes', 'hex', or 'string'".to_string(),
                required: false,
                default: Some(Value::String("hex".to_string())),
                example: Some(Value::String("string".to_string())),
                enum_values: Some(vec!["bytes".to_string(), "hex".to_string(), "string".to_string()]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "generate_random",
            "parameters": {
                "length": 16,
                "format": "hex"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Random hex: 7f83b1657ff1fc53b92dc18148a1d65d".to_string();
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
        debug!("Executing generate_random driver");
        let length = parameters.get("length").and_then(|v| v.as_u64()).ok_or_else(|| {
            debug!("Missing 'length' parameter");
            DriverError::missing_parameter("length")
        })? as usize;
        let format = parameters.get("format").and_then(|v| v.as_str()).unwrap_or("hex");
        debug!("Random generation parameters: length={}, format={}", length, format);
        if length == 0 {
            warn!("Length is zero");
            return Err(DriverError::validation("length", "Length must be greater than 0"));
        }
        let result = match format {
            "bytes" => {
                let bytes = generate_random_bytes(length).map_err(|e| DriverError::execution(format!("Failed to generate random bytes: {}", e)))?;
                format!("Random bytes: {:?}", bytes)
            }
            "hex" => {
                let hex = generate_random_hex(length).map_err(|e| DriverError::execution(format!("Failed to generate random hex: {}", e)))?;
                format!("Random hex: {}", hex)
            }
            "string" => {
                let string =
                    generate_random_string(length).map_err(|e| DriverError::execution(format!("Failed to generate random string: {}", e)))?;
                format!("Random string: {}", string)
            }
            _ => {
                warn!("Unsupported format: {}", format);
                return Err(DriverError::execution(format!("Unsupported format: {}", format)));
            }
        };
        info!("Random data generated successfully");
        return Ok(result);
    }
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating generate_random parameters");
        if parameters.get("length").is_none() {
            return Err(DriverError::missing_parameter("length"));
        }
        debug!("Parameter validation passed");
        return Ok(());
    }
}
