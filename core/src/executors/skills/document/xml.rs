use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::{file_exists, read_file_content, types::{Skill, SkillParameter}, validate_path};

#[derive(Debug)]
pub struct XmlParseSkill;

#[async_trait::async_trait]
impl Skill for XmlParseSkill {
    fn name(&self) -> &str {
        "xml_parse"
    }

    fn description(&self) -> &str {
        "Parse XML content from a file or string"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to parse XML configuration files, extract data from XML documents, or read XML feeds"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "XML content as string OR path to XML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("<root><name>test</name></root>".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "is_path".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether source is a file path (true) or raw XML string (false)"
                    .to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "xpath".to_string(),
                param_type: "string".to_string(),
                description: "XPath expression to extract specific nodes (optional)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("//name".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "xml_parse",
            "parameters": {
                "source": "<data><item>value</item></data>"
            }
        })
    }

    fn example_output(&self) -> String {
        "{\"data\": {\"item\": \"value\"}}".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let source = parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source' parameter"))?;
        let is_path = parameters
            .get("is_path")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let xpath = parameters.get("xpath").and_then(|v| v.as_str());
        let xml_content = if is_path {
            let validated_path = validate_path(source, None)?;
            if !file_exists(&validated_path.to_string_lossy()) {
                anyhow::bail!("XML file not found: {}", source);
            }
            read_file_content(&validated_path.to_string_lossy())?
        } else {
            source.to_string()
        };
        let result = xml_to_json(&xml_content, xpath)?;
        Ok(result)
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: source"))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct XmlToJsonSkill;

#[async_trait::async_trait]
impl Skill for XmlToJsonSkill {
    fn name(&self) -> &str {
        "xml_to_json"
    }

    fn description(&self) -> &str {
        "Convert XML content to JSON format"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to convert XML data to JSON for easier processing"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "source".to_string(),
                param_type: "string".to_string(),
                description: "XML content as string OR path to XML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("<root><name>test</name></root>".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "is_path".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether source is a file path (true) or raw XML string (false)"
                    .to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "pretty".to_string(),
                param_type: "boolean".to_string(),
                description: "Pretty-print the JSON output".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(false)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "xml_to_json",
            "parameters": {
                "source": "<person><name>Alice</name><age>30</age></person>"
            }
        })
    }

    fn example_output(&self) -> String {
        "{\n  \"person\": {\n    \"name\": \"Alice\",\n    \"age\": 30\n  }\n}".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let source = parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'source' parameter"))?;
        let is_path = parameters
            .get("is_path")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let pretty = parameters
            .get("pretty")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let xml_content = if is_path {
            let validated_path = validate_path(source, None)?;
            if !file_exists(&validated_path.to_string_lossy()) {
                anyhow::bail!("XML file not found: {}", source);
            }
            read_file_content(&validated_path.to_string_lossy())?
        } else {
            source.to_string()
        };
        let json_value = xml_to_json_value(&xml_content)?;
        if pretty {
            Ok(serde_json::to_string_pretty(&json_value)?)
        } else {
            Ok(serde_json::to_string(&json_value)?)
        }
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: source"))?;
        Ok(())
    }
}

// Helper function for simple XML to JSON conversion
fn xml_to_json(xml: &str, xpath: Option<&str>) -> Result<String> {
    let value = xml_to_json_value(xml)?;
    Ok(serde_json::to_string_pretty(&value)?)
}

fn xml_to_json_value(xml: &str) -> Result<serde_json::Value> {
    use quick_xml::de::from_str;
    let result: Result<serde_json::Value, _> = from_str(xml);
    match result {
        Ok(value) => Ok(value),
        Err(e) => Ok(json!({
            "error": format!("Failed to parse XML: {}", e),
            "raw_content": xml
        })),
    }
}
