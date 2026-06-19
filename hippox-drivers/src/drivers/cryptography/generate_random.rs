//! Random generation driver

use super::common::{generate_random_bytes, generate_random_hex, generate_random_string};
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Driver for generating random data
///
/// # Description
/// Generates cryptographically secure random data in various formats: bytes, hex, or string.
///
/// # Parameters
/// * `length` (required) - Length of random data to generate
/// * `format` (optional) - "bytes" (default), "hex", or "string"
///
/// # Example
/// ```
/// Input: length=16, format="hex"
/// Output: "Random hex: 7f83b1657ff1fc53b92dc18148a1d65d"
/// ```
#[derive(Debug)]
pub struct GenerateRandomDriver;

#[async_trait::async_trait]
impl Driver for GenerateRandomDriver {
    fn name(&self) -> &str {
        "generate_random"
    }

    fn description(&self) -> &str {
        "Generate cryptographically secure random data (bytes, hex, or string)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need random data for keys, tokens, or testing purposes."
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
                enum_values: Some(vec![
                    "bytes".to_string(),
                    "hex".to_string(),
                    "string".to_string(),
                ]),
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "generate_random",
            "parameters": {
                "length": 16,
                "format": "hex"
            }
        })
    }

    fn example_output(&self) -> String {
        "Random hex: 7f83b1657ff1fc53b92dc18148a1d65d".to_string()
    }

    fn category(&self) -> crate::DriverCategory {
        crate::DriverCategory::Cryptography
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let length = parameters
            .get("length")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'length' parameter"))?
            as usize;
        let format = parameters
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("hex");

        if length == 0 {
            anyhow::bail!("Length must be greater than 0");
        }

        let result = match format {
            "bytes" => {
                let bytes = generate_random_bytes(length)?;
                format!("Random bytes: {:?}", bytes)
            }
            "hex" => {
                let hex = generate_random_hex(length)?;
                format!("Random hex: {}", hex)
            }
            "string" => {
                let string = generate_random_string(length)?;
                format!("Random string: {}", string)
            }
            _ => anyhow::bail!("Unsupported format: {}", format),
        };

        Ok(result)
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("length")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: length"))?;
        Ok(())
    }
}
