//! JSON file driver module
//!
//! This module provides drivers for JSON file operations including
//! reading JSON files, parsing JSON data, writing JSON files,
//! validating JSON syntax, and querying JSON data.
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{DriverError, DriverResult, ensure_dir, file_exists, read_file_content, validate_path, write_file_content};
use serde_json::{Value, from_str, json, to_string_pretty};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for reading JSON files
#[derive(Debug)]
pub struct JsonReadDriver;
#[async_trait::async_trait]
impl Driver for JsonReadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "json_read";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Read and parse JSON file content";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to read a JSON file, parse configuration data, or extract structured data from .json files";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the JSON file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("config.json".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "Optional JSONPath query to extract specific data (e.g., '$.users[0].name')".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("$.data.results".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "pretty".to_string(),
                param_type: "boolean".to_string(),
                description: "Pretty-print the JSON output".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "json_read",
            "parameters": {
                "path": "config.json"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "{\n  \"name\": \"example\",\n  \"version\": \"1.0\"\n}".to_string();
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
        debug!("Executing json_read driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let query = parameters.get("query").and_then(|v| v.as_str());
        let pretty = parameters.get("pretty").and_then(|v| v.as_bool()).unwrap_or(true);
        debug!("Reading JSON file: {}, query: {:?}", path, query);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("JSON file not found: {}", path)));
        }
        let content =
            read_file_content(&validated_path.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to read file: {}", e)))?;
        let json_value: Value = from_str(&content).map_err(|e| DriverError::execution(format!("Failed to parse JSON: {}", e)))?;
        let result = if let Some(q) = query {
            let extracted = query_json(&json_value, q)?;
            if pretty {
                to_string_pretty(&extracted).map_err(|e| DriverError::execution(format!("Failed to serialize JSON: {}", e)))?
            } else {
                extracted.to_string()
            }
        } else if pretty {
            to_string_pretty(&json_value).map_err(|e| DriverError::execution(format!("Failed to serialize JSON: {}", e)))?
        } else {
            json_value.to_string()
        };
        info!("JSON read completed: {}", path);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Driver for writing JSON files
#[derive(Debug)]
pub struct JsonWriteDriver;
#[async_trait::async_trait]
impl Driver for JsonWriteDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "json_write";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Write data to JSON file";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to create, save, or update a JSON file with structured data";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the JSON file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("output.json".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "data".to_string(),
                param_type: "object".to_string(),
                description: "JSON data to write (can be object or array)".to_string(),
                required: true,
                default: None,
                example: Some(json!({"name": "example", "value": 42})),
                enum_values: None,
            },
            DriverParameter {
                name: "pretty".to_string(),
                param_type: "boolean".to_string(),
                description: "Pretty-print the JSON output".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "merge".to_string(),
                param_type: "boolean".to_string(),
                description: "Merge with existing JSON file (only works with objects)".to_string(),
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
            "action": "json_write",
            "parameters": {
                "path": "output.json",
                "data": {"name": "example", "version": "1.0"}
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "JSON written to: output.json".to_string();
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
        debug!("Executing json_write driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let data = parameters.get("data").ok_or_else(|| DriverError::missing_parameter("data"))?;
        let pretty = parameters.get("pretty").and_then(|v| v.as_bool()).unwrap_or(true);
        let merge = parameters.get("merge").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Writing JSON file: {}, merge: {}", path, merge);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to create directory: {}", e)))?;
        }
        let final_data = if merge && file_exists(&validated_path.to_string_lossy()) {
            let existing_content = read_file_content(&validated_path.to_string_lossy())
                .map_err(|e| DriverError::execution(format!("Failed to read existing file: {}", e)))?;
            let existing_json: Value =
                from_str(&existing_content).map_err(|e| DriverError::execution(format!("Failed to parse existing JSON: {}", e)))?;
            merge_json(&existing_json, data)?
        } else {
            data.clone()
        };
        let content = if pretty {
            to_string_pretty(&final_data).map_err(|e| DriverError::execution(format!("Failed to serialize JSON: {}", e)))?
        } else {
            final_data.to_string()
        };
        write_file_content(&validated_path.to_string_lossy(), &content, false)
            .map_err(|e| DriverError::execution(format!("Failed to write file: {}", e)))?;
        info!("JSON written to: {}", path);
        return Ok(format!("JSON written to: {}", path));
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        parameters.get("data").ok_or_else(|| DriverError::missing_parameter("data"))?;
        return Ok(());
    }
}
/// Driver for validating JSON syntax
#[derive(Debug)]
pub struct JsonValidateDriver;
#[async_trait::async_trait]
impl Driver for JsonValidateDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "json_validate";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Validate JSON file syntax and optional schema validation";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to check if a JSON file is valid or validate against a schema";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the JSON file to validate".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("data.json".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "schema_path".to_string(),
                param_type: "string".to_string(),
                description: "Optional path to JSON Schema file for validation".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("schema.json".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "json_validate",
            "parameters": {
                "path": "data.json"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "JSON is valid".to_string();
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
        debug!("Executing json_validate driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        debug!("Validating JSON file: {}", path);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("JSON file not found: {}", path)));
        }
        let content =
            read_file_content(&validated_path.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to read file: {}", e)))?;
        match from_str::<Value>(&content) {
            Ok(json_value) => {
                let mut output = format!("JSON is valid\n");
                output.push_str(&format!("  Type: {}\n", json_type_name(&json_value)));
                if let Some(schema_path) = parameters.get("schema_path").and_then(|v| v.as_str()) {
                    debug!("Validating against schema: {}", schema_path);
                    let schema_validated_path =
                        validate_path(schema_path, None).map_err(|e| DriverError::execution(format!("Invalid schema path: {}", e)))?;
                    if file_exists(&schema_validated_path.to_string_lossy()) {
                        let schema_content = read_file_content(&schema_validated_path.to_string_lossy())
                            .map_err(|e| DriverError::execution(format!("Failed to read schema file: {}", e)))?;
                        let schema: Value =
                            from_str(&schema_content).map_err(|e| DriverError::execution(format!("Failed to parse schema: {}", e)))?;
                        validate_json_schema(&json_value, &schema)?;
                        output.push_str("  Schema validation passed\n");
                        info!("Schema validation passed for: {}", path);
                    } else {
                        output.push_str(&format!("  Schema file not found: {}\n", schema_path));
                        info!("Schema file not found: {}", schema_path);
                    }
                }
                info!("JSON validation completed: {}", path);
                return Ok(output);
            }
            Err(e) => {
                return Err(DriverError::execution(format!("Invalid JSON: {}", e)));
            }
        }
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Queries JSON data using a simple dot notation path
///
/// # Arguments
/// * `value` - JSON value to query
/// * `query` - Query path (e.g., "$.data.results" or "$.users[0].name")
///
/// # Returns
/// * `DriverResult<Value>` - Extracted JSON value
fn query_json(value: &Value, query: &str) -> DriverResult<Value> {
    if query == "$" || query == "." {
        return Ok(value.clone());
    }
    let parts: Vec<&str> = query.split('.').collect();
    let mut current = value;
    for part in parts {
        if part.starts_with('$') {
            let key = &part[1..];
            if !key.is_empty() {
                current = current.get(key).ok_or_else(|| DriverError::execution(format!("Path '{}' not found", query)))?;
            }
        } else if part.starts_with('[') && part.ends_with(']') {
            let idx_str = &part[1..part.len() - 1];
            if let Ok(idx) = idx_str.parse::<usize>() {
                if let Some(arr) = current.as_array() {
                    current = arr.get(idx).ok_or_else(|| DriverError::execution(format!("Index {} out of range", idx)))?;
                } else {
                    return Err(DriverError::execution(format!("Cannot index non-array with [{}]", idx_str)));
                }
            } else {
                return Err(DriverError::execution(format!("Invalid array index: {}", idx_str)));
            }
        } else {
            current = current.get(part).ok_or_else(|| DriverError::execution(format!("Path '{}' not found", query)))?;
        }
    }
    return Ok(current.clone());
}
/// Merges two JSON values
///
/// # Arguments
/// * `existing` - Existing JSON value
/// * `new` - New JSON value to merge
///
/// # Returns
/// * `DriverResult<Value>` - Merged JSON value
fn merge_json(existing: &Value, new: &Value) -> DriverResult<Value> {
    match (existing, new) {
        (Value::Object(existing_obj), Value::Object(new_obj)) => {
            let mut merged = existing_obj.clone();
            for (k, v) in new_obj {
                if let Some(existing_val) = merged.get(k) {
                    merged.insert(k.clone(), merge_json(existing_val, v)?);
                } else {
                    merged.insert(k.clone(), v.clone());
                }
            }
            return Ok(Value::Object(merged));
        }
        _ => return Ok(new.clone()),
    }
}
/// Gets the JSON type name as a string
///
/// # Arguments
/// * `value` - JSON value
///
/// # Returns
/// * `&'static str` - Type name
fn json_type_name(value: &Value) -> &'static str {
    return match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    };
}
/// Validates JSON against a simple schema
///
/// # Arguments
/// * `value` - JSON value to validate
/// * `schema` - Schema to validate against
///
/// # Returns
/// * `DriverResult<()>` - Ok if valid, Err otherwise
fn validate_json_schema(value: &Value, schema: &Value) -> DriverResult<()> {
    // Check required fields
    if let Some(required) = schema.get("required").and_then(|v| v.as_array()) {
        if let Some(obj) = value.as_object() {
            for req in required {
                if let Some(req_str) = req.as_str() {
                    if !obj.contains_key(req_str) {
                        return Err(DriverError::validation("schema", format!("Missing required field: {}", req_str)));
                    }
                }
            }
        }
    }
    // Check property types
    if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
        if let Some(obj) = value.as_object() {
            for (prop_name, prop_schema) in properties {
                if let Some(prop_value) = obj.get(prop_name) {
                    validate_property(prop_value, prop_schema, prop_name)?;
                }
            }
        }
    }
    return Ok(());
}
/// Validates a single property against its schema
///
/// # Arguments
/// * `value` - Property value
/// * `schema` - Property schema
/// * `prop_name` - Property name for error messages
///
/// # Returns
/// * `DriverResult<()>` - Ok if valid, Err otherwise
fn validate_property(value: &Value, schema: &Value, prop_name: &str) -> DriverResult<()> {
    if let Some(expected_type) = schema.get("type").and_then(|v| v.as_str()) {
        let actual_type = json_type_name(value);
        if actual_type != expected_type {
            return Err(DriverError::validation(prop_name, format!("Expected type '{}' but got '{}'", expected_type, actual_type)));
        }
    }
    return Ok(());
}
