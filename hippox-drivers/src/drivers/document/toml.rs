//! TOML file driver module
//!
//! This module provides drivers for TOML file operations including
//! reading TOML files, parsing TOML data, writing TOML files,
//! and validating TOML syntax.
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{DriverError, DriverResult, ensure_dir, file_exists, read_file_content, validate_path, write_file_content};
/// Driver for reading TOML files
#[derive(Debug)]
pub struct TomlReadDriver;
#[async_trait::async_trait]
impl Driver for TomlReadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "toml_read";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Read and parse TOML file content";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to read a TOML file, parse configuration data (like Cargo.toml), or extract content from .toml files";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the TOML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Cargo.toml".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "to_json".to_string(),
                param_type: "boolean".to_string(),
                description: "Convert TOML to JSON format".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "toml_read",
            "parameters": {
                "path": "Cargo.toml"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "[package]\nname = \"myapp\"\nversion = \"0.1.0\"".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Document;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing toml_read driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let to_json = parameters.get("to_json").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Reading TOML file: {}, to_json: {}", path, to_json);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("TOML file not found: {}", path)));
        }
        let content =
            read_file_content(&validated_path.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to read file: {}", e)))?;
        let toml_value: toml::Value = toml::from_str(&content).map_err(|e| DriverError::execution(format!("Failed to parse TOML: {}", e)))?;
        if to_json {
            let json_value =
                serde_json::to_value(&toml_value).map_err(|e| DriverError::execution(format!("Failed to convert TOML to JSON: {}", e)))?;
            let result = serde_json::to_string_pretty(&json_value).map_err(|e| DriverError::execution(format!("Failed to serialize JSON: {}", e)))?;
            info!("TOML read completed and converted to JSON: {}", path);
            return Ok(result);
        } else {
            info!("TOML read completed: {}", path);
            return Ok(content);
        }
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Driver for writing TOML files
#[derive(Debug)]
pub struct TomlWriteDriver;
#[async_trait::async_trait]
impl Driver for TomlWriteDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "toml_write";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Write data to TOML file";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to create, save, or update a TOML file with configuration data";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the TOML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("config.toml".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "data".to_string(),
                param_type: "object".to_string(),
                description: "Data to write (as JSON object that will be converted to TOML)".to_string(),
                required: true,
                default: None,
                example: Some(json!({"package": {"name": "myapp", "version": "0.1.0"}})),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "toml_write",
            "parameters": {
                "path": "config.toml",
                "data": {"package": {"name": "myapp", "version": "0.1.0"}}
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "TOML written to: config.toml".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Document;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing toml_write driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let data = parameters.get("data").ok_or_else(|| DriverError::missing_parameter("data"))?;
        debug!("Writing TOML file: {}", path);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to create directory: {}", e)))?;
        }
        let json_value = serde_json::to_value(data).map_err(|e| DriverError::execution(format!("Failed to serialize data: {}", e)))?;
        let toml_value: toml::Value =
            serde_json::from_value(json_value).map_err(|e| DriverError::execution(format!("Failed to convert JSON to TOML: {}", e)))?;
        let toml_content = toml::to_string(&toml_value).map_err(|e| DriverError::execution(format!("Failed to serialize TOML: {}", e)))?;
        write_file_content(&validated_path.to_string_lossy(), &toml_content, false)
            .map_err(|e| DriverError::execution(format!("Failed to write file: {}", e)))?;
        info!("TOML written to: {}", path);
        return Ok(format!("TOML written to: {}", path));
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        parameters.get("data").ok_or_else(|| DriverError::missing_parameter("data"))?;
        return Ok(());
    }
}
/// Driver for validating TOML syntax
#[derive(Debug)]
pub struct TomlValidateDriver;
#[async_trait::async_trait]
impl Driver for TomlValidateDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "toml_validate";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Validate TOML file syntax";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to check if a TOML file has valid syntax";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the TOML file to validate".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Cargo.toml".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "toml_validate",
            "parameters": {
                "path": "Cargo.toml"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "TOML is valid".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Document;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing toml_validate driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        debug!("Validating TOML file: {}", path);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("TOML file not found: {}", path)));
        }
        let content =
            read_file_content(&validated_path.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to read file: {}", e)))?;
        match toml::from_str::<toml::Value>(&content) {
            Ok(_) => {
                info!("TOML is valid: {}", path);
                return Ok("TOML is valid".to_string());
            }
            Err(e) => {
                return Err(DriverError::execution(format!("Invalid TOML: {}", e)));
            }
        }
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
