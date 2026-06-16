use anyhow::Result;
use serde_json::{Value, from_str, json, to_string_pretty};
use std::collections::HashMap;

use crate::{
    SkillCategory, ensure_dir, file_exists, read_file_content,
    types::{Skill, SkillParameter},
    validate_path, write_file_content,
};

#[derive(Debug)]
pub struct JsonReadSkill;

#[async_trait::async_trait]
impl Skill for JsonReadSkill {
    fn name(&self) -> &str {
        "json_read"
    }

    fn description(&self) -> &str {
        "Read and parse JSON file content"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read a JSON file, parse configuration data, or extract structured data from .json files"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the JSON file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("config.json".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description:
                    "Optional JSONPath query to extract specific data (e.g., '$.users[0].name')"
                        .to_string(),
                required: false,
                default: None,
                example: Some(Value::String("$.data.results".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "pretty".to_string(),
                param_type: "boolean".to_string(),
                description: "Pretty-print the JSON output".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "json_read",
            "parameters": {
                "path": "config.json"
            }
        })
    }

    fn example_output(&self) -> String {
        "{\n  \"name\": \"example\",\n  \"version\": \"1.0\"\n}".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Document
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let query = parameters.get("query").and_then(|v| v.as_str());
        let pretty = parameters
            .get("pretty")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("JSON file not found: {}", path);
        }
        let content = read_file_content(&validated_path.to_string_lossy())?;
        let json_value: Value = from_str(&content)?;
        let result = if let Some(q) = query {
            let extracted = query_json(&json_value, q)?;
            if pretty {
                to_string_pretty(&extracted)?
            } else {
                extracted.to_string()
            }
        } else if pretty {
            to_string_pretty(&json_value)?
        } else {
            json_value.to_string()
        };
        Ok(result)
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct JsonWriteSkill;

#[async_trait::async_trait]
impl Skill for JsonWriteSkill {
    fn name(&self) -> &str {
        "json_write"
    }

    fn description(&self) -> &str {
        "Write data to JSON file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to create, save, or update a JSON file with structured data"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the JSON file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("output.json".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "data".to_string(),
                param_type: "object".to_string(),
                description: "JSON data to write (can be object or array)".to_string(),
                required: true,
                default: None,
                example: Some(json!({"name": "example", "value": 42})),
                enum_values: None,
            },
            SkillParameter {
                name: "pretty".to_string(),
                param_type: "boolean".to_string(),
                description: "Pretty-print the JSON output".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "merge".to_string(),
                param_type: "boolean".to_string(),
                description: "Merge with existing JSON file (only works with objects)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "json_write",
            "parameters": {
                "path": "output.json",
                "data": {"name": "example", "version": "1.0"}
            }
        })
    }

    fn example_output(&self) -> String {
        "JSON written to: output.json".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Document
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let data = parameters
            .get("data")
            .ok_or_else(|| anyhow::anyhow!("Missing 'data' parameter"))?;
        let pretty = parameters
            .get("pretty")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let merge = parameters
            .get("merge")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let validated_path = validate_path(path, None)?;
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy())?;
        }

        let final_data = if merge && file_exists(&validated_path.to_string_lossy()) {
            let existing_content = read_file_content(&validated_path.to_string_lossy())?;
            let existing_json: Value = from_str(&existing_content)?;
            merge_json(&existing_json, data)?
        } else {
            data.clone()
        };

        let content = if pretty {
            to_string_pretty(&final_data)?
        } else {
            final_data.to_string()
        };

        write_file_content(&validated_path.to_string_lossy(), &content, false)?;
        Ok(format!("JSON written to: {}", path))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        parameters
            .get("data")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: data"))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct JsonValidateSkill;

#[async_trait::async_trait]
impl Skill for JsonValidateSkill {
    fn name(&self) -> &str {
        "json_validate"
    }

    fn description(&self) -> &str {
        "Validate JSON file syntax and optional schema validation"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to check if a JSON file is valid or validate against a schema"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the JSON file to validate".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("data.json".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "schema_path".to_string(),
                param_type: "string".to_string(),
                description: "Optional path to JSON Schema file for validation".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("schema.json".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "json_validate",
            "parameters": {
                "path": "data.json"
            }
        })
    }

    fn example_output(&self) -> String {
        "JSON is valid".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Document
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("JSON file not found: {}", path);
        }
        let content = read_file_content(&validated_path.to_string_lossy())?;
        match from_str::<Value>(&content) {
            Ok(json_value) => {
                let mut output = format!("✓ JSON is valid\n");
                output.push_str(&format!("  Type: {}\n", json_type_name(&json_value)));
                if let Some(schema_path) = parameters.get("schema_path").and_then(|v| v.as_str()) {
                    let schema_validated_path = validate_path(schema_path, None)?;
                    if file_exists(&schema_validated_path.to_string_lossy()) {
                        let schema_content =
                            read_file_content(&schema_validated_path.to_string_lossy())?;
                        let schema: Value = from_str(&schema_content)?;
                        if let Err(e) = validate_json_schema(&json_value, &schema) {
                            anyhow::bail!("Schema validation failed: {}", e);
                        }
                        output.push_str("  ✓ Schema validation passed\n");
                    } else {
                        output.push_str(&format!("  ⚠ Schema file not found: {}\n", schema_path));
                    }
                }

                Ok(output)
            }
            Err(e) => anyhow::bail!("Invalid JSON: {}", e),
        }
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        Ok(())
    }
}

fn query_json(value: &Value, query: &str) -> Result<Value> {
    if query == "$" || query == "." {
        return Ok(value.clone());
    }

    let parts: Vec<&str> = query.split('.').collect();
    let mut current = value;

    for part in parts {
        if part.starts_with('$') {
            let key = &part[1..];
            if !key.is_empty() {
                current = current
                    .get(key)
                    .ok_or_else(|| anyhow::anyhow!("Path '{}' not found", query))?;
            }
        } else if part.starts_with('[') && part.ends_with(']') {
            let idx_str = &part[1..part.len() - 1];
            if let Ok(idx) = idx_str.parse::<usize>() {
                if let Some(arr) = current.as_array() {
                    current = arr
                        .get(idx)
                        .ok_or_else(|| anyhow::anyhow!("Index {} out of range", idx))?;
                } else {
                    anyhow::bail!("Cannot index non-array with [{}]", idx);
                }
            } else {
                anyhow::bail!("Invalid array index: {}", idx_str);
            }
        } else {
            current = current
                .get(part)
                .ok_or_else(|| anyhow::anyhow!("Path '{}' not found", query))?;
        }
    }

    Ok(current.clone())
}

fn merge_json(existing: &Value, new: &Value) -> Result<Value> {
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
            Ok(Value::Object(merged))
        }
        _ => Ok(new.clone()),
    }
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn validate_json_schema(value: &Value, schema: &Value) -> Result<()> {
    if let Some(required) = schema.get("required").and_then(|v| v.as_array()) {
        if let Some(obj) = value.as_object() {
            for req in required {
                if let Some(req_str) = req.as_str() {
                    if !obj.contains_key(req_str) {
                        anyhow::bail!("Missing required field: {}", req_str);
                    }
                }
            }
        }
    }
    if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
        if let Some(obj) = value.as_object() {
            for (prop_name, prop_schema) in properties {
                if let Some(prop_value) = obj.get(prop_name) {
                    validate_property(prop_value, prop_schema, prop_name)?;
                }
            }
        }
    }
    Ok(())
}

fn validate_property(value: &Value, schema: &Value, prop_name: &str) -> Result<()> {
    if let Some(expected_type) = schema.get("type").and_then(|v| v.as_str()) {
        let actual_type = json_type_name(value);
        if actual_type != expected_type {
            anyhow::bail!(
                "Property '{}' expected type '{}' but got '{}'",
                prop_name,
                expected_type,
                actual_type
            );
        }
    }
    Ok(())
}
