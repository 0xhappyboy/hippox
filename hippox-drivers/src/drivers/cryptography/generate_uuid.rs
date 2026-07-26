//! UUID generation driver
//!
//! This driver provides functionality to generate universally unique identifiers.
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{DriverError, DriverResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;
/// Driver for generating UUID
///
/// Generates a universally unique identifier (UUID) version 4 (random).
#[derive(Debug)]
pub struct GenerateUuidDriver;
#[async_trait::async_trait]
impl Driver for GenerateUuidDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "generate_uuid"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Generate a UUID (version 4)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when you need a unique identifier for resources, sessions, or tracking."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "format".to_string(),
                param_type: "string".to_string(),
                description: "Output format: 'hyphenated', 'simple', 'braced', or 'urn'".to_string(),
                required: false,
                default: Some(Value::String("hyphenated".to_string())),
                example: Some(Value::String("simple".to_string())),
                enum_values: Some(vec!["hyphenated".to_string(), "simple".to_string(), "braced".to_string(), "urn".to_string()]),
            },
            DriverParameter {
                name: "count".to_string(),
                param_type: "integer".to_string(),
                description: "Number of UUIDs to generate (default: 1)".to_string(),
                required: false,
                default: Some(Value::Number(1.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "generate_uuid",
            "parameters": {
                "format": "hyphenated"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "UUID: 550e8400-e29b-41d4-a716-446655440000".to_string();
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
        debug!("Executing generate_uuid driver");
        let format = parameters.get("format").and_then(|v| v.as_str()).unwrap_or("hyphenated");
        let count = parameters.get("count").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        debug!("UUID generation parameters: format={}, count={}", format, count);
        if count == 0 {
            warn!("Count is zero");
            return Err(DriverError::validation("count", "Count must be greater than 0"));
        }
        let mut uuids = Vec::new();
        for i in 0..count {
            debug!("Generating UUID {}/{}", i + 1, count);
            let uuid = Uuid::new_v4();
            let formatted = match format {
                "simple" => uuid.simple().to_string(),
                "braced" => uuid.braced().to_string(),
                "urn" => uuid.urn().to_string(),
                _ => uuid.hyphenated().to_string(),
            };
            uuids.push(formatted);
        }
        info!("Generated {} UUID(s)", uuids.len());
        if uuids.len() == 1 {
            return Ok(format!("UUID: {}", uuids[0]));
        } else {
            let mut output = String::from("UUIDs:\n");
            for (i, uuid) in uuids.iter().enumerate() {
                output.push_str(&format!("{}. {}\n", i + 1, uuid));
            }
            return Ok(output);
        }
    }
    /// Validate parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        debug!("Validating generate_uuid parameters");
        debug!("Parameter validation passed");
        return Ok(());
    }
}
