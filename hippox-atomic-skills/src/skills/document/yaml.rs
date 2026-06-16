use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use crate::{ensure_dir, file_exists, read_file_content, validate_path, write_file_content};

#[derive(Debug)]
pub struct YamlReadSkill;

#[async_trait::async_trait]
impl Skill for YamlReadSkill {
    fn name(&self) -> &str {
        "yaml_read"
    }

    fn description(&self) -> &str {
        "Read and parse YAML file content"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read a YAML file, parse configuration data, or extract content from .yml/.yaml files"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the YAML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("config.yml".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "to_json".to_string(),
                param_type: "boolean".to_string(),
                description: "Convert YAML to JSON format".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "yaml_read",
            "parameters": {
                "path": "config.yml"
            }
        })
    }

    fn example_output(&self) -> String {
        "name: example\nversion: 1.0".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Document
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let to_json = parameters
            .get("to_json")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("YAML file not found: {}", path);
        }

        let content = read_file_content(&validated_path.to_string_lossy())?;
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;

        if to_json {
            let json_value = serde_json::to_value(&yaml_value)?;
            Ok(serde_json::to_string_pretty(&json_value)?)
        } else {
            Ok(content)
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

#[derive(Debug)]
pub struct YamlWriteSkill;

#[async_trait::async_trait]
impl Skill for YamlWriteSkill {
    fn name(&self) -> &str {
        "yaml_write"
    }

    fn description(&self) -> &str {
        "Write data to YAML file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to create, save, or update a YAML file with structured data"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the YAML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("output.yml".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "data".to_string(),
                param_type: "object".to_string(),
                description: "Data to write (can be provided as YAML string or JSON object)"
                    .to_string(),
                required: true,
                default: None,
                example: Some(json!({"name": "example", "value": 42})),
                enum_values: None,
            },
            SkillParameter {
                name: "data_format".to_string(),
                param_type: "string".to_string(),
                description: "Format of the data parameter ('json' or 'yaml')".to_string(),
                required: false,
                default: Some(Value::String("json".to_string())),
                example: Some(Value::String("yaml".to_string())),
                enum_values: Some(vec!["json".to_string(), "yaml".to_string()]),
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "yaml_write",
            "parameters": {
                "path": "output.yml",
                "data": {"name": "example", "version": "1.0"}
            }
        })
    }

    fn example_output(&self) -> String {
        "YAML written to: output.yml".to_string()
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
        let data_format = parameters
            .get("data_format")
            .and_then(|v| v.as_str())
            .unwrap_or("json");
        let validated_path = validate_path(path, None)?;
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy())?;
        }
        let yaml_content = if data_format == "yaml" {
            if let Some(yaml_str) = data.as_str() {
                serde_yaml::from_str::<serde_yaml::Value>(yaml_str)?;
                yaml_str.to_string()
            } else {
                anyhow::bail!("Data must be a string when format is 'yaml'");
            }
        } else {
            let json_value = serde_json::to_value(data)?;
            serde_yaml::to_string(&json_value)?
        };
        write_file_content(&validated_path.to_string_lossy(), &yaml_content, false)?;
        Ok(format!("YAML written to: {}", path))
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
pub struct YamlValidateSkill;

#[async_trait::async_trait]
impl Skill for YamlValidateSkill {
    fn name(&self) -> &str {
        "yaml_validate"
    }

    fn description(&self) -> &str {
        "Validate YAML file syntax"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to check if a YAML file has valid syntax"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the YAML file to validate".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("config.yml".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "yaml_validate",
            "parameters": {
                "path": "config.yml"
            }
        })
    }

    fn example_output(&self) -> String {
        "YAML is valid".to_string()
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
            anyhow::bail!("YAML file not found: {}", path);
        }
        let content = read_file_content(&validated_path.to_string_lossy())?;
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
                Ok(format!("YAML is valid\n  Root type: {}", type_name))
            }
            Err(e) => anyhow::bail!("Invalid YAML: {}", e),
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
