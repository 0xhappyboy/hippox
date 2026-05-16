use crate::executors::{SkillCall, registry};
use anyhow::Result;
use serde_json::Value;

/// Executor is responsible for parsing LLM responses and executing skills
///
/// It follows this workflow:
/// 1. Parse JSON response from LLM into SkillCall
/// 2. Look up the skill by name in the registry
/// 3. Execute the skill with the provided parameters
#[derive(Debug, Clone)]
pub struct Executor;

impl Executor {
    /// Create a new Executor instance
    pub fn new() -> Self {
        Self
    }

    /// Parse a JSON string into a SkillCall
    ///
    /// # Arguments
    /// * `json_str` - JSON string from LLM response, e.g., `{"action": "helloworld", "parameters": {"name": "Alice"}}`
    pub fn parse_skill_call(&self, json_str: &str) -> Result<SkillCall> {
        Ok(serde_json::from_str(json_str)?)
    }

    /// Parse a JSON Value into a SkillCall
    ///
    /// # Arguments
    /// * `json_value` - JSON Value object from LLM response
    pub fn parse_skill_call_from_value(&self, json_value: &Value) -> Result<SkillCall> {
        Ok(serde_json::from_value(json_value.clone())?)
    }

    /// Execute a skill based on the SkillCall
    ///
    /// # Arguments
    /// * `call` - The parsed skill call containing action name and parameters
    ///
    /// # Returns
    /// The result string from skill execution
    pub async fn execute(&self, call: &SkillCall) -> Result<String> {
        let skill = registry::get_skill(&call.action)
            .ok_or_else(|| anyhow::anyhow!("Unknown skill: {}", call.action))?;
        skill.execute(&call.parameters).await
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_execute_helloworld_from_llm_json() {
        let executor = Executor::new();
        // Simulate JSON response from LLM
        let llm_response = r#"{"action": "helloworld", "parameters": {"name": "Alice"}}"#;
        // Parse the LLM response into SkillCall
        let call = executor.parse_skill_call(llm_response).unwrap();
        // Execute the skill
        let result = executor.execute(&call).await.unwrap();
        assert_eq!(result, "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_execute_helloworld_from_llm_json_without_parameters() {
        let executor = Executor::new();
        // Simulate JSON response from LLM without parameters
        let llm_response = r#"{"action": "helloworld"}"#;
        let call = executor.parse_skill_call(llm_response).unwrap();
        let result = executor.execute(&call).await.unwrap();
        // Default value "World" should be used when no name parameter is provided
        assert_eq!(result, "Hello, World!");
    }

    #[tokio::test]
    async fn test_unknown_skill_from_llm() {
        let executor = Executor::new();
        // Simulate LLM returning a non-existent skill
        let llm_response = r#"{"action": "nonexistent_skill", "parameters": {}}"#;
        let call = executor.parse_skill_call(llm_response).unwrap();
        let result = executor.execute(&call).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown skill"));
    }

    #[tokio::test]
    async fn test_invalid_json_from_llm() {
        let executor = Executor::new();
        // Simulate LLM returning invalid JSON
        let invalid_json = "not a json";
        let result = executor.parse_skill_call(invalid_json);
        assert!(result.is_err());
    }
}
