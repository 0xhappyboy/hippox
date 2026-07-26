//! YAML file driver module
//!
//! This module provides drivers for YAML file operations including
//! reading YAML files, parsing YAML data, writing YAML files,
//! and validating YAML syntax.
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
/// Driver for reading YAML files
#[derive(Debug)]
pub struct YamlReadDriver;
#[async_trait::async_trait]
impl Driver for YamlReadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "yaml_read";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Read and parse YAML file content";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to read a YAML file, parse configuration data, or extract content from .yml/.yaml files";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the YAML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("config.yml".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "to_json".to_string(),
                param_type: "boolean".to_string(),
                description: "Convert YAML to JSON format".to_string(),
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
            "action": "yaml_read",
            "parameters": {
                "path": "config.yml"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "name: example\nversion: 1.0".to_string();
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
        debug!("Executing yaml_read driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let to_json = parameters.get("to_json").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Reading YAML file: {}, to_json: {}", path, to_json);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("YAML file not found: {}", path)));
        }
        let content =
            read_file_content(&validated_path.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to read file: {}", e)))?;
        let yaml_value: serde_yaml::Value =
            serde_yaml::from_str(&content).map_err(|e| DriverError::execution(format!("Failed to parse YAML: {}", e)))?;
        if to_json {
            let json_value =
                serde_json::to_value(&yaml_value).map_err(|e| DriverError::execution(format!("Failed to convert YAML to JSON: {}", e)))?;
            let result = serde_json::to_string_pretty(&json_value).map_err(|e| DriverError::execution(format!("Failed to serialize JSON: {}", e)))?;
            info!("YAML read completed and converted to JSON: {}", path);
            return Ok(result);
        } else {
            info!("YAML read completed: {}", path);
            return Ok(content);
        }
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Driver for writing YAML files
#[derive(Debug)]
pub struct YamlWriteDriver;
#[async_trait::async_trait]
impl Driver for YamlWriteDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "yaml_write";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Write data to YAML file";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to create, save, or update a YAML file with structured data";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the YAML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("output.yml".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "data".to_string(),
                param_type: "object".to_string(),
                description: "Data to write (can be provided as YAML string or JSON object)".to_string(),
                required: true,
                default: None,
                example: Some(json!({"name": "example", "value": 42})),
                enum_values: None,
            },
            DriverParameter {
                name: "data_format".to_string(),
                param_type: "string".to_string(),
                description: "Format of the data parameter ('json' or 'yaml')".to_string(),
                required: false,
                default: Some(Value::String("json".to_string())),
                example: Some(Value::String("yaml".to_string())),
                enum_values: Some(vec!["json".to_string(), "yaml".to_string()]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "yaml_write",
            "parameters": {
                "path": "output.yml",
                "data": {"name": "example", "version": "1.0"}
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "YAML written to: output.yml".to_string();
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
        debug!("Executing yaml_write driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let data = parameters.get("data").ok_or_else(|| DriverError::missing_parameter("data"))?;
        let data_format = parameters.get("data_format").and_then(|v| v.as_str()).unwrap_or("json");
        debug!("Writing YAML file: {}, data_format: {}", path, data_format);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to create directory: {}", e)))?;
        }
        let yaml_content = if data_format == "yaml" {
            if let Some(yaml_str) = data.as_str() {
                // Validate YAML syntax
                serde_yaml::from_str::<serde_yaml::Value>(yaml_str).map_err(|e| DriverError::execution(format!("Invalid YAML: {}", e)))?;
                yaml_str.to_string()
            } else {
                return Err(DriverError::execution("Data must be a string when format is 'yaml'".to_string()));
            }
        } else {
            let json_value = serde_json::to_value(data).map_err(|e| DriverError::execution(format!("Failed to serialize data: {}", e)))?;
            serde_yaml::to_string(&json_value).map_err(|e| DriverError::execution(format!("Failed to convert to YAML: {}", e)))?
        };
        write_file_content(&validated_path.to_string_lossy(), &yaml_content, false)
            .map_err(|e| DriverError::execution(format!("Failed to write file: {}", e)))?;
        info!("YAML written to: {}", path);
        return Ok(format!("YAML written to: {}", path));
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        parameters.get("data").ok_or_else(|| DriverError::missing_parameter("data"))?;
        return Ok(());
    }
}
/// Driver for validating YAML syntax
#[derive(Debug)]
pub struct YamlValidateDriver;
#[async_trait::async_trait]
impl Driver for YamlValidateDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "yaml_validate";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Validate YAML file syntax";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to check if a YAML file has valid syntax";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the YAML file to validate".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("config.yml".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "yaml_validate",
            "parameters": {
                "path": "config.yml"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "YAML is valid".to_string();
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
        debug!("Executing yaml_validate driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        debug!("Validating YAML file: {}", path);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("YAML file not found: {}", path)));
        }
        let content =
            read_file_content(&validated_path.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to read file: {}", e)))?;
        match serde_yaml::from_str::<serde_yaml::Value>(&content) {
            Ok(yaml_value) => {
                let type_name = match yaml_value {
                    serde_yaml::Value::Null => "null",
                    serde_yaml::Value::Bool(_) => "boolean",
                    serde_yaml::Value::Number(_) => "number",
                    serde_yaml::Value::String(_) => "string",
                    serde_yaml::Value::Sequence(_) => "sequence/array",
                    serde_yaml::Value::Mapping(_) => "mapping/object",
                    serde_yaml::Value::Tagged(_) => "tagged",
                };
                info!("YAML is valid: {} (type: {})", path, type_name);
                return Ok(format!("YAML is valid\n  Root type: {}", type_name));
            }
            Err(e) => {
                return Err(DriverError::execution(format!("Invalid YAML: {}", e)));
            }
        }
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
