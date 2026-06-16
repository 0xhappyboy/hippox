use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use crate::{ensure_dir, file_exists, read_file_content, validate_path, write_file_content};

#[derive(Debug)]
pub struct TomlReadSkill;

#[async_trait::async_trait]
impl Skill for TomlReadSkill {
    fn name(&self) -> &str {
        "toml_read"
    }

    fn description(&self) -> &str {
        "Read and parse TOML file content"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read a TOML file, parse configuration data (like Cargo.toml), or extract content from .toml files"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the TOML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Cargo.toml".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "to_json".to_string(),
                param_type: "boolean".to_string(),
                description: "Convert TOML to JSON format".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "toml_read",
            "parameters": {
                "path": "Cargo.toml"
            }
        })
    }

    fn example_output(&self) -> String {
        "[package]\nname = \"myapp\"\nversion = \"0.1.0\"".to_string()
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
            anyhow::bail!("TOML file not found: {}", path);
        }

        let content = read_file_content(&validated_path.to_string_lossy())?;
        let toml_value: toml::Value = toml::from_str(&content)?;

        if to_json {
            let json_value = serde_json::to_value(&toml_value)?;
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
pub struct TomlWriteSkill;

#[async_trait::async_trait]
impl Skill for TomlWriteSkill {
    fn name(&self) -> &str {
        "toml_write"
    }

    fn description(&self) -> &str {
        "Write data to TOML file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to create, save, or update a TOML file with configuration data"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the TOML file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("config.toml".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "data".to_string(),
                param_type: "object".to_string(),
                description: "Data to write (as JSON object that will be converted to TOML)"
                    .to_string(),
                required: true,
                default: None,
                example: Some(json!({"package": {"name": "myapp", "version": "0.1.0"}})),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "toml_write",
            "parameters": {
                "path": "config.toml",
                "data": {"package": {"name": "myapp", "version": "0.1.0"}}
            }
        })
    }

    fn example_output(&self) -> String {
        "TOML written to: config.toml".to_string()
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

        let validated_path = validate_path(path, None)?;
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy())?;
        }

        let json_value = serde_json::to_value(data)?;
        let toml_value: toml::Value = serde_json::from_value(json_value)?;
        let toml_content = toml::to_string(&toml_value)?;

        write_file_content(&validated_path.to_string_lossy(), &toml_content, false)?;
        Ok(format!("TOML written to: {}", path))
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
pub struct TomlValidateSkill;

#[async_trait::async_trait]
impl Skill for TomlValidateSkill {
    fn name(&self) -> &str {
        "toml_validate"
    }

    fn description(&self) -> &str {
        "Validate TOML file syntax"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to check if a TOML file has valid syntax"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the TOML file to validate".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Cargo.toml".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "toml_validate",
            "parameters": {
                "path": "Cargo.toml"
            }
        })
    }

    fn example_output(&self) -> String {
        "TOML is valid".to_string()
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
            anyhow::bail!("TOML file not found: {}", path);
        }
        let content = read_file_content(&validated_path.to_string_lossy())?;
        match toml::from_str::<toml::Value>(&content) {
            Ok(_) => Ok("TOML is valid".to_string()),
            Err(e) => anyhow::bail!("Invalid TOML: {}", e),
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
