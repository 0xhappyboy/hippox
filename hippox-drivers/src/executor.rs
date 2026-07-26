use std::sync::Arc;

/// # Executor Module
///
/// This module provides the `Executor` struct, which is responsible for parsing
/// Large Language Model (LLM) responses and executing corresponding drivers.
///
/// ## Overview
///
/// The executor follows a simple workflow:
/// 1. Parse a JSON response from an LLM into a structured `DriverCall`
/// 2. Look up the requested Driver by name in the global registry
/// 3. Execute the Driver with the provided parameters
///
/// ## Example
///
/// ```rust,ignore
/// use executor::Executor;
///
/// let executor = Executor::new();
/// let llm_response = r#"{"action": "helloworld", "parameters": {"name": "Alice"}}"#;
/// let call = executor.parse_driver_call(llm_response)?;
/// let result = executor.execute(&call).await?;
/// println!("{}", result); // "Hello, Alice!"
/// ```
///
/// ## Error Handling
///
/// The executor returns `DriverResult` types, with errors occurring in three scenarios:
/// - Invalid JSON input during parsing
/// - Unknown Driver name (not found in registry)
/// - Driver execution failure (delegated to the Driver itself)
use crate::{DriverCall, DriverCallback, DriverContext, DriverError, DriverResult, get_driver_by_name};
use serde_json::Value;

/// Executor is responsible for parsing LLM responses and executing skills
///
/// The `Executor` acts as the bridge between LLM outputs and the skill execution system.
/// It validates, parses, and routes skill calls to their appropriate implementations.
///
/// # Workflow
///
/// 1. **Parse JSON response from LLM into DriverCall**
///    - Accepts either a JSON string or a pre-parsed `serde_json::Value`
///    - Validates the structure matches `DriverCall` (action + optional parameters)
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
/// let call = executor.parse_driver_call(r#"{"action": "calculate", "parameters": {"a": 5, "b": 3}}"#)?;
/// let result = executor.execute(&call).await?;
///
/// // From JSON Value
/// let json_value = json!({"action": "greet", "parameters": {"name": "Bob"}});
/// let call = executor.parse_driver_call_from_value(&json_value)?;
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

    /// Parse a JSON string into a DriverCall
    ///
    /// This method deserializes a JSON string into a `DriverCall` structure.
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
    /// * `DriverResult<DriverCall>` - The parsed driver call, or an error if parsing fails
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
    /// let call = executor.parse_driver_call(r#"{"action": "send_email", "parameters": {"to": "user@example.com"}}"#)?;
    /// assert_eq!(call.action, "send_email");
    ///
    /// // Valid JSON without parameters
    /// let call = executor.parse_driver_call(r#"{"action": "ping"}"#)?;
    /// assert_eq!(call.action, "ping");
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_driver_call(&self, json_str: &str) -> DriverResult<DriverCall> {
        serde_json::from_str(json_str).map_err(|e| DriverError::Internal { message: format!("Failed to parse JSON: {}", e) })
    }

    /// Parse a JSON Value into a DriverCall
    ///
    /// This is an alternative to `parse_driver_call` that accepts a pre-parsed
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
    /// * `DriverResult<DriverCall>` - The parsed driver call, or an error if parsing fails
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
    /// let call = executor.parse_driver_call_from_value(&json_value)?;
    /// assert_eq!(call.action, "process_data");
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_driver_call_from_value(&self, json_value: &Value) -> DriverResult<DriverCall> {
        serde_json::from_value(json_value.clone()).map_err(|e| DriverError::Internal { message: format!("Failed to parse JSON value: {}", e) })
    }

    /// Execute a driver based on the DriverCall
    ///
    /// This method performs the actual driver execution by:
    /// 1. Looking up the driver in the global registry by its action name
    /// 2. If found, calling the driver's `execute` method with the parameters
    ///
    /// # Arguments
    ///
    /// * `call` - The parsed driver call containing action name and parameters
    ///
    /// # Returns
    ///
    /// * `DriverResult<String>` - The result string from driver execution
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The requested driver does not exist in the registry
    /// - The driver's execution fails (error is propagated from the driver)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use anyhow::Result;
    /// # use executor::Executor;
    /// # use crate::executors::DriverCall;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let executor = Executor::new();
    /// let call = DriverCall {
    ///     action: "helloworld".to_string(),
    ///     parameters: Some(json!({"name": "Alice"})),
    /// };
    ///
    /// let result = executor.execute(&call).await?;
    /// println!("Execution result: {}", result);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute(&self, call: &DriverCall, callback: Option<&dyn DriverCallback>, context: Option<&DriverContext>) -> DriverResult<String> {
        let driver = get_driver_by_name(&call.action).ok_or_else(|| DriverError::DriverNotFound { name: call.action.clone() })?;
        driver.execute(&call.parameters, callback, context).await
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
    use std::collections::HashMap;

    use super::*;
    use serde_json::json;

    // Note: These tests require the driver registry to be initialized with
    // the helloworld driver. They are kept as examples but may need to be
    // adapted based on the actual driver registry setup.

    #[tokio::test]
    async fn test_parse_driver_call() {
        let executor = Executor::new();
        let json_str = r#"{"action": "test_action", "parameters": {"key": "value"}}"#;
        let call = executor.parse_driver_call(json_str).unwrap();
        assert_eq!(call.action, "test_action");
        assert_eq!(call.parameters.get("key").unwrap().as_str(), Some("value"));
    }

    #[tokio::test]
    async fn test_parse_driver_call_without_parameters() {
        let executor = Executor::new();
        let json_str = r#"{"action": "test_action"}"#;
        let call = executor.parse_driver_call(json_str).unwrap();
        assert_eq!(call.action, "test_action");
        assert!(call.parameters.is_empty());
    }

    #[tokio::test]
    async fn test_parse_driver_call_from_value() {
        let executor = Executor::new();
        let json_value = json!({
            "action": "test_action",
            "parameters": {
                "key": "value"
            }
        });
        let call = executor.parse_driver_call_from_value(&json_value).unwrap();
        assert_eq!(call.action, "test_action");
        assert_eq!(call.parameters.get("key").unwrap().as_str(), Some("value"));
    }

    #[tokio::test]
    async fn test_parse_driver_call_from_value_without_parameters() {
        let executor = Executor::new();
        let json_value = json!({
            "action": "test_action"
        });
        let call = executor.parse_driver_call_from_value(&json_value).unwrap();
        assert_eq!(call.action, "test_action");
        assert!(call.parameters.is_empty());
    }

    #[tokio::test]
    async fn test_invalid_json_parse() {
        let executor = Executor::new();
        let invalid_json = "not a json";
        let result = executor.parse_driver_call(invalid_json);
        assert!(result.is_err());
        match result {
            Err(DriverError::Internal { message }) => {
                assert!(message.contains("Failed to parse JSON"));
            }
            _ => panic!("Expected Internal error"),
        }
    }

    #[tokio::test]
    async fn test_driver_not_found() {
        let executor = Executor::new();
        let call = DriverCall { action: "nonexistent_driver".to_string(), parameters: HashMap::new() };
        let result = executor.execute(&call, None, None).await;
        assert!(result.is_err());
        match result {
            Err(DriverError::DriverNotFound { name }) => {
                assert_eq!(name, "nonexistent_driver");
            }
            _ => panic!("Expected DriverNotFound error"),
        }
    }
}
