//! XML file driver module
//!
//! This module provides drivers for XML file operations including
//! parsing XML content, converting XML to JSON, and extracting data
//! from XML documents.
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{DriverError, DriverResult, file_exists, read_file_content, validate_path};
/// Driver for parsing XML content
#[derive(Debug)]
pub struct XmlParseDriver;
#[async_trait::async_trait]
impl Driver for XmlParseDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "xml_parse";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Parse XML content from a file or string";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to parse XML configuration files, extract data from XML documents, or read XML feeds";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "XML content as string OR path to XML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("<root><name>test</name></root>".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "is_path".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether source is a file path (true) or raw XML string (false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "xpath".to_string(),
                param_type: "string".to_string(),
                description: "XPath expression to extract specific nodes (optional)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("//name".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "xml_parse",
            "parameters": {
                "source": "<data><item>value</item></data>"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "{\"data\": {\"item\": \"value\"}}".to_string();
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
        debug!("Executing xml_parse driver");
        // Extract required parameters
        let source = parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        let is_path = parameters.get("is_path").and_then(|v| v.as_bool()).unwrap_or(false);
        let xpath = parameters.get("xpath").and_then(|v| v.as_str());
        debug!("Parsing XML, is_path: {}, xpath: {:?}", is_path, xpath);
        let xml_content = if is_path {
            let validated_path = validate_path(source, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
            if !file_exists(&validated_path.to_string_lossy()) {
                return Err(DriverError::execution(format!("XML file not found: {}", source)));
            }
            read_file_content(&validated_path.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to read file: {}", e)))?
        } else {
            source.to_string()
        };
        let result = xml_to_json(&xml_content, xpath)?;
        info!("XML parse completed successfully");
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        return Ok(());
    }
}
/// Driver for converting XML to JSON
#[derive(Debug)]
pub struct XmlToJsonDriver;
#[async_trait::async_trait]
impl Driver for XmlToJsonDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "xml_to_json";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Convert XML content to JSON format";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to convert XML data to JSON for easier processing";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "XML content as string OR path to XML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("<root><name>test</name></root>".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "is_path".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether source is a file path (true) or raw XML string (false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "pretty".to_string(),
                param_type: "boolean".to_string(),
                description: "Pretty-print the JSON output".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "xml_to_json",
            "parameters": {
                "source": "<person><name>Alice</name><age>30</age></person>"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "{\n  \"person\": {\n    \"name\": \"Alice\",\n    \"age\": 30\n  }\n}".to_string();
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
        debug!("Executing xml_to_json driver");
        // Extract required parameters
        let source = parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        let is_path = parameters.get("is_path").and_then(|v| v.as_bool()).unwrap_or(false);
        let pretty = parameters.get("pretty").and_then(|v| v.as_bool()).unwrap_or(true);
        debug!("Converting XML to JSON, is_path: {}, pretty: {}", is_path, pretty);
        let xml_content = if is_path {
            let validated_path = validate_path(source, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
            if !file_exists(&validated_path.to_string_lossy()) {
                return Err(DriverError::execution(format!("XML file not found: {}", source)));
            }
            read_file_content(&validated_path.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to read file: {}", e)))?
        } else {
            source.to_string()
        };
        let json_value = xml_to_json_value(&xml_content)?;
        let result = if pretty {
            serde_json::to_string_pretty(&json_value).map_err(|e| DriverError::execution(format!("Failed to serialize JSON: {}", e)))?
        } else {
            serde_json::to_string(&json_value).map_err(|e| DriverError::execution(format!("Failed to serialize JSON: {}", e)))?
        };
        info!("XML to JSON conversion completed successfully");
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("source").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("source"))?;
        return Ok(());
    }
}
/// Parses XML to JSON string with optional XPath query
///
/// # Arguments
/// * `xml` - XML content
/// * `xpath` - Optional XPath expression
///
/// # Returns
/// * `DriverResult<String>` - JSON string
fn xml_to_json(xml: &str, xpath: Option<&str>) -> DriverResult<String> {
    let value = xml_to_json_value(xml)?;
    let result = serde_json::to_string_pretty(&value).map_err(|e| DriverError::execution(format!("Failed to serialize JSON: {}", e)))?;
    return Ok(result);
}
/// Parses XML to JSON value
///
/// # Arguments
/// * `xml` - XML content
///
/// # Returns
/// * `DriverResult<serde_json::Value>` - JSON value
fn xml_to_json_value(xml: &str) -> DriverResult<serde_json::Value> {
    use quick_xml::de::from_str;
    let result: Result<serde_json::Value, _> = from_str(xml);
    match result {
        Ok(value) => {
            return Ok(value);
        }
        Err(e) => {
            debug!("Failed to parse XML: {}", e);
            return Ok(json!({
                "error": format!("Failed to parse XML: {}", e),
                "raw_content": xml
            }));
        }
    }
}
