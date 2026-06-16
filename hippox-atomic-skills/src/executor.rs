use std::sync::Arc;

/// # Executor Module
///
/// This module provides the `Executor` struct, which is responsible for parsing
/// Large Language Model (LLM) responses and executing corresponding skills.
///
/// ## Overview
///
/// The executor follows a simple workflow:
/// 1. Parse a JSON response from an LLM into a structured `SkillCall`
/// 2. Look up the requested skill by name in the global registry
/// 3. Execute the skill with the provided parameters
///
/// ## Example
///
/// ```rust,ignore
/// use executor::Executor;
///
/// let executor = Executor::new();
/// let llm_response = r#"{"action": "helloworld", "parameters": {"name": "Alice"}}"#;
/// let call = executor.parse_skill_call(llm_response)?;
/// let result = executor.execute(&call).await?;
/// println!("{}", result); // "Hello, Alice!"
/// ```
///
/// ## Error Handling
///
/// The executor returns `anyhow::Result` types, with errors occurring in three scenarios:
/// - Invalid JSON input during parsing
/// - Unknown skill name (not found in registry)
/// - Skill execution failure (delegated to the skill itself)
use crate::{Skill, SkillCall, get_registry, get_skill_by_name};
use anyhow::Result;
use serde_json::Value;

/// Executor is responsible for parsing LLM responses and executing skills
///
/// The `Executor` acts as the bridge between LLM outputs and the skill execution system.
/// It validates, parses, and routes skill calls to their appropriate implementations.
///
/// # Workflow
///
/// 1. **Parse JSON response from LLM into SkillCall**
///    - Accepts either a JSON string or a pre-parsed `serde_json::Value`
///    - Validates the structure matches `SkillCall` (action + optional parameters)
///
/// 2. **Look up the skill by name in the registry**
///    - Uses the global skill registry to find the requested skill
///    - Returns an error if the skill doesn't exist
///
/// 3. **Execute the skill with the provided parameters**
///    - Delegates execution to the skill's `execute` method
///    - Passes the parameters as a `serde_json::Value`
///
/// # Thread Safety
///
/// `Executor` is `Send` and `Sync` because it contains no internal state.
/// It can be safely shared across async tasks and threads.
///
/// # Example
///
/// ```rust,no_run
/// # use anyhow::Result;
/// # use executor::Executor;
/// # use serde_json::json;
/// # #[tokio::main]
/// # async fn main() -> Result<()> {
/// let executor = Executor::new();
///
/// // From JSON string
/// let call = executor.parse_skill_call(r#"{"action": "calculate", "parameters": {"a": 5, "b": 3}}"#)?;
/// let result = executor.execute(&call).await?;
///
/// // From JSON Value
/// let json_value = json!({"action": "greet", "parameters": {"name": "Bob"}});
/// let call = executor.parse_skill_call_from_value(&json_value)?;
/// let result = executor.execute(&call).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Executor;

impl Executor {
    /// Create a new Executor instance
    ///
    /// This constructor creates an executor with default configuration.
    /// Since the executor is stateless, multiple instances are functionally identical.
    ///
    /// # Returns
    ///
    /// A new `Executor` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use executor::Executor;
    /// let executor = Executor::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Parse a JSON string into a SkillCall
    ///
    /// This method deserializes a JSON string into a `SkillCall` structure.
    /// It expects the JSON to have an "action" field and an optional "parameters" field.
    ///
    /// # Arguments
    ///
    /// * `json_str` - JSON string from LLM response
    ///   - Format: `{"action": "skill_name", "parameters": {...}}`
    ///   - The `parameters` field is optional
    ///
    /// # Returns
    ///
    /// * `Result<SkillCall>` - The parsed skill call, or an error if parsing fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The JSON is malformed (invalid syntax)
    /// - The JSON is missing the required "action" field
    /// - The "action" field is not a string
    ///
    /// # Example
    ///
    /// ```rust
    /// # use anyhow::Result;
    /// # use executor::Executor;
    /// # fn main() -> Result<()> {
    /// let executor = Executor::new();
    ///
    /// // Valid JSON with parameters
    /// let call = executor.parse_skill_call(r#"{"action": "send_email", "parameters": {"to": "user@example.com"}}"#)?;
    /// assert_eq!(call.action, "send_email");
    ///
    /// // Valid JSON without parameters
    /// let call = executor.parse_skill_call(r#"{"action": "ping"}"#)?;
    /// assert_eq!(call.action, "ping");
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_skill_call(&self, json_str: &str) -> Result<SkillCall> {
        Ok(serde_json::from_str(json_str)?)
    }

    /// Parse a JSON Value into a SkillCall
    ///
    /// This is an alternative to `parse_skill_call` that accepts a pre-parsed
    /// `serde_json::Value`. This is useful when you already have a Value object
    /// from another parsing operation or when working with streaming JSON.
    ///
    /// # Arguments
    ///
    /// * `json_value` - JSON Value object from LLM response
    ///   - Should contain "action" and optionally "parameters" fields
    ///
    /// # Returns
    ///
    /// * `Result<SkillCall>` - The parsed skill call, or an error if parsing fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The Value is missing the required "action" field
    /// - The "action" field is not a string
    /// - The structure doesn't match the expected format
    ///
    /// # Example
    ///
    /// ```rust
    /// # use anyhow::Result;
    /// # use executor::Executor;
    /// # use serde_json::json;
    /// # fn main() -> Result<()> {
    /// let executor = Executor::new();
    /// let json_value = json!({
    ///     "action": "process_data",
    ///     "parameters": {
    ///         "input": "some data",
    ///         "format": "json"
    ///     }
    /// });
    ///
    /// let call = executor.parse_skill_call_from_value(&json_value)?;
    /// assert_eq!(call.action, "process_data");
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_skill_call_from_value(&self, json_value: &Value) -> Result<SkillCall> {
        Ok(serde_json::from_value(json_value.clone())?)
    }

    /// Execute a skill based on the SkillCall
    ///
    /// This method performs the actual skill execution by:
    /// 1. Looking up the skill in the global registry by its action name
    /// 2. If found, calling the skill's `execute` method with the parameters
    ///
    /// # Arguments
    ///
    /// * `call` - The parsed skill call containing action name and parameters
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The result string from skill execution
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The requested skill does not exist in the registry
    /// - The skill's execution fails (error is propagated from the skill)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use anyhow::Result;
    /// # use executor::Executor;
    /// # use crate::executors::SkillCall;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let executor = Executor::new();
    /// let call = SkillCall {
    ///     action: "helloworld".to_string(),
    ///     parameters: Some(json!({"name": "Alice"})),
    /// };
    ///
    /// let result = executor.execute(&call).await?;
    /// println!("Execution result: {}", result);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute(&self, call: &SkillCall) -> Result<String> {
        let skill = get_skill_by_name(&call.action)
            .ok_or_else(|| anyhow::anyhow!("Unknown skill: {}", call.action))?;
        skill.execute(&call.parameters).await
    }
}

impl Default for Executor {
    /// Creates a default Executor instance
    ///
    /// This implementation simply calls `Executor::new()` since the executor
    /// requires no configuration.
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
        let llm_response = r#"{"action": "helloworld", "parameters": {"name": "Alice"}}"#;
        let call = executor.parse_skill_call(llm_response).unwrap();
        let result = executor.execute(&call).await.unwrap();
        assert_eq!(result, "Hello, Alice!");
    }

    #[tokio::test]
    async fn test_execute_helloworld_from_llm_json_without_parameters() {
        let executor = Executor::new();
        let llm_response = r#"{"action": "helloworld"}"#;
        let call = executor.parse_skill_call(llm_response).unwrap();
        let result = executor.execute(&call).await.unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[tokio::test]
    async fn test_unknown_skill_from_llm() {
        let executor = Executor::new();
        let llm_response = r#"{"action": "nonexistent_skill", "parameters": {}}"#;
        let call = executor.parse_skill_call(llm_response).unwrap();
        let result = executor.execute(&call).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown skill"));
    }

    #[tokio::test]
    async fn test_invalid_json_from_llm() {
        let executor = Executor::new();
        let invalid_json = "not a json";
        let result = executor.parse_skill_call(invalid_json);
        assert!(result.is_err());
    }

    /// Test that parse_skill_call_from_value correctly parses a JSON Value
    /// This test verifies the alternative parsing method works identically to
    /// string-based parsing when given valid input.
    #[tokio::test]
    async fn test_parse_skill_call_from_value() {
        let executor = Executor::new();
        let json_value = json!({
            "action": "helloworld",
            "parameters": {
                "name": "TestUser"
            }
        });
        let call = executor.parse_skill_call_from_value(&json_value).unwrap();
        assert_eq!(call.action, "helloworld");
        assert!(!call.parameters.is_empty());
        let result = executor.execute(&call).await.unwrap();
        assert_eq!(result, "Hello, TestUser!");
    }

    /// Test that parse_skill_call_from_value handles missing parameters field correctly
    /// This test ensures that the parser gracefully handles the absence of the
    /// optional "parameters" field in the JSON input.
    #[tokio::test]
    async fn test_parse_skill_call_from_value_without_parameters() {
        let executor = Executor::new();
        let json_value = json!({
            "action": "helloworld"
        });
        let call = executor.parse_skill_call_from_value(&json_value).unwrap();
        assert_eq!(call.action, "helloworld");
        assert!(call.parameters.is_empty());
        let result = executor.execute(&call).await.unwrap();
        assert_eq!(result, "Hello, World!");
    }

    /// Test that parse_skill_call handles empty parameters object correctly
    /// This test verifies that an empty parameters object `{}` is properly
    /// deserialized as Some(Value::Object({})) rather than None.
    #[tokio::test]
    async fn test_parse_skill_call_with_empty_parameters_object() {
        let executor = Executor::new();
        let llm_response = r#"{"action": "helloworld", "parameters": {}}"#;
        let call = executor.parse_skill_call(llm_response).unwrap();
        assert_eq!(call.action, "helloworld");
        assert!(!call.parameters.is_empty());
        let result = executor.execute(&call).await.unwrap();
        assert_eq!(result, "Hello, World!");
    }
}
