//! File signature verification skill
//!
//! This driver provides functionality to verify file signatures
//! using SHA256 hash-based integrity checking.
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{file_exists, validate_path, verify_file_signature};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for verifying file signatures
#[derive(Debug)]
pub struct FileSignatureDriver;
#[async_trait::async_trait]
impl Driver for FileSignatureDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "file_signature_verify"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Verify file signature (hash-based integrity check)"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to verify if a file matches an expected signature/hash."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the file to verify".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/file.txt".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "signature".to_string(),
                param_type: "string".to_string(),
                description: "Expected SHA256 signature/hash".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "file_signature_verify",
            "parameters": {
                "path": "/tmp/file.txt",
                "signature": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "File signature verified: /tmp/file.txt matches expected signature".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::File;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing file_signature_verify driver");
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'path' parameter");
            return crate::DriverError::missing_parameter("path");
        })?;
        let signature = parameters.get("signature").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'signature' parameter");
            return crate::DriverError::missing_parameter("signature");
        })?;
        debug!("Verifying signature for {}: {}", path, &signature[..16]);
        let validated_path = validate_path(path, None).map_err(|e| {
            debug!("Failed to validate path: {}", e);
            return crate::DriverError::execution(format!("Failed to validate path: {}", e));
        })?;
        if !file_exists(&validated_path.to_string_lossy()) {
            warn!("File not found: {}", path);
            return Err(crate::DriverError::execution(format!("File not found: {}", path)));
        }
        let verified = verify_file_signature(&validated_path.to_string_lossy(), signature).map_err(|e| {
            debug!("Failed to verify signature: {}", e);
            return crate::DriverError::execution(format!("Failed to verify signature: {}", e));
        })?;
        if verified {
            info!("File signature verified: {}", path);
            return Ok(format!("File signature verified: {} matches expected signature", path));
        } else {
            warn!("File signature mismatch: {}", path);
            return Ok(format!("File signature mismatch: {} does NOT match expected signature", path));
        }
    }
}
