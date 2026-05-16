use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;

/// Skill parameter definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillParameter {
    /// Parameter name (used in JSON call)
    pub name: String,
    /// Parameter type (string, integer, boolean, array, object)
    #[serde(rename = "type")]
    pub param_type: String,
    /// Human-readable description of what this parameter does
    pub description: String,
    /// Whether this parameter must be provided
    #[serde(default)]
    pub required: bool,
    /// Default value if not provided
    #[serde(default)]
    pub default: Option<Value>,
    /// Example value to help LLM understand format
    #[serde(default)]
    pub example: Option<Value>,
    /// Possible values (for enums/limited options)
    #[serde(default)]
    pub enum_values: Option<Vec<String>>,
}

/// Complete skill metadata for LLM
#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillMetadata {
    /// Skill name (used as action in SkillCall)
    pub name: String,
    /// Brief description of what this skill does
    pub description: String,
    /// Detailed explanation of when to use this skill
    pub usage_hint: String,
    /// Parameter definitions
    pub parameters: Vec<SkillParameter>,
    /// Example of how to call this skill (JSON format)
    pub example_call: serde_json::Value,
    /// Example output format
    pub example_output: String,
    /// Category for grouping (file, net, math, time, system)
    pub category: String,
}

/// Skill execution trait
#[async_trait::async_trait]
pub trait Skill: Send + Sync + Debug {
    /// Skill name (must match the `name` field in SKILL.md)
    fn name(&self) -> &str;
    /// Brief description (one line)
    fn description(&self) -> &str;
    /// Detailed usage hint
    fn usage_hint(&self) -> &str {
        "No usage hint provided"
    }
    /// Parameter definitions
    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }
    /// Example call (JSON format)
    fn example_call(&self) -> serde_json::Value {
        serde_json::json!({})
    }
    /// Example output
    fn example_output(&self) -> String {
        String::new()
    }
    /// Category (file, net, math, time, system)
    fn category(&self) -> &str {
        "general"
    }
    /// Execute the skill with given parameters
    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String>;
    /// Validate parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        Ok(())
    }
    /// Get full metadata for LLM
    fn get_metadata(&self) -> SkillMetadata {
        SkillMetadata {
            name: self.name().to_string(),
            description: self.description().to_string(),
            usage_hint: self.usage_hint().to_string(),
            parameters: self.parameters(),
            example_call: self.example_call(),
            example_output: self.example_output(),
            category: self.category().to_string(),
        }
    }
}

/// Skill call instruction parsed from LLM response
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SkillCall {
    pub action: String,
    #[serde(default)]
    pub parameters: HashMap<String, Value>,
}
