use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::SkillCategory;

/// Skill parameter definition
///
/// Defines a single parameter that can be passed to a skill. This includes
/// type information, validation constraints, and documentation to help LLMs
/// understand how to use the parameter correctly.
///
/// # Examples
///
/// ```rust
/// # use serde_json::json;
/// # use your_crate::SkillParameter;
/// let param = SkillParameter {
///     name: "timeout".to_string(),
///     param_type: "integer".to_string(),
///     description: "Maximum time in seconds to wait".to_string(),
///     required: false,
///     default: Some(json!(30)),
///     example: Some(json!(60)),
///     enum_values: None,
/// };
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillParameter {
    /// Parameter name (used in JSON call)
    ///
    /// This name must be used as the key when providing the parameter
    /// in a skill call's parameters map.
    pub name: String,

    /// Parameter type (string, integer, boolean, array, object)
    ///
    /// Supported types:
    /// - `"string"`: Text values
    /// - `"integer"`: Whole numbers
    /// - `"boolean"`: true/false values
    /// - `"array"`: JSON array
    /// - `"object"`: JSON object
    #[serde(rename = "type")]
    pub param_type: String,

    /// Human-readable description of what this parameter does
    ///
    /// This description is provided to the LLM to help it understand
    /// the purpose and appropriate values for this parameter.
    pub description: String,

    /// Whether this parameter must be provided
    ///
    /// If `true`, the skill will fail validation if this parameter
    /// is missing from the call. If `false`, the skill should use
    /// the `default` value or handle absence gracefully.
    #[serde(default)]
    pub required: bool,

    /// Default value if not provided
    ///
    /// Used when the parameter is optional (`required: false`) and
    /// the caller does not supply a value.
    #[serde(default)]
    pub default: Option<Value>,

    /// Example value to help LLM understand format
    ///
    /// Provides a concrete example that demonstrates the expected
    /// format and typical values for this parameter.
    #[serde(default)]
    pub example: Option<Value>,

    /// Possible values (for enums/limited options)
    ///
    /// When provided, the parameter value must be one of the strings
    /// in this list. This is useful for parameters that accept only
    /// a predefined set of options.
    #[serde(default)]
    pub enum_values: Option<Vec<String>>,
}

/// Complete skill metadata for LLM
///
/// Contains all information about a skill that is needed by the LLM
/// to understand when and how to invoke the skill. This metadata is
/// typically serialized to JSON and included in the system prompt
/// or tool definitions.
///
/// # Example
///
/// ```rust
/// # use serde_json::json;
/// # use your_crate::{SkillMetadata, SkillParameter};
/// let metadata = SkillMetadata {
///     name: "read_file".to_string(),
///     description: "Read contents of a file".to_string(),
///     usage_hint: "Use when you need to examine file contents".to_string(),
///     parameters: vec![
///         SkillParameter {
///             name: "path".to_string(),
///             param_type: "string".to_string(),
///             description: "File path to read".to_string(),
///             required: true,
///             default: None,
///             example: Some(json!("./docs/readme.md")),
///             enum_values: None,
///         }
///     ],
///     example_call: json!({"action": "read_file", "parameters": {"path": "./config.json"}}),
///     example_output: "File contents as string".to_string(),
///     category: "file".to_string(),
/// };
/// ```
#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillMetadata {
    /// Skill name (used as action in SkillCall)
    ///
    /// This name must match the `action` field in a `SkillCall` to
    /// invoke this skill.
    pub name: String,

    /// Brief description of what this skill does
    ///
    /// One-sentence summary that helps the LLM quickly identify
    /// whether this skill is appropriate for the current task.
    pub description: String,

    /// Detailed explanation of when to use this skill
    ///
    /// Provides guidance on the specific scenarios where this skill
    /// should be invoked, including any prerequisites or context
    /// requirements.
    pub usage_hint: String,

    /// Parameter definitions
    ///
    /// Complete list of parameters that this skill accepts, including
    /// type information, validation rules, and documentation.
    pub parameters: Vec<SkillParameter>,

    /// Example of how to call this skill (JSON format)
    ///
    /// A concrete example showing the expected JSON structure for
    /// invoking this skill, including parameter names and example values.
    pub example_call: serde_json::Value,

    /// Example output format
    ///
    /// A string describing or demonstrating the expected output format
    /// so the LLM knows what to expect when the skill returns.
    pub example_output: String,

    /// Category for grouping (file, net, math, time, system)
    ///
    /// Used to organize skills by functional area. Common categories:
    /// - `"file"`: File system operations
    /// - `"net"`: Network/HTTP operations
    /// - `"math"`: Mathematical computations
    /// - `"time"`: Time/date operations
    /// - `"system"`: System-level operations
    pub category: SkillCategory,
}

/// Atomic skill execution context
///
/// Passed from the external layer to atomic skills. Skills are read-only consumers
/// and do not know the implementation details of the upper layer.
#[derive(Debug, Clone, Default)]
pub struct SkillContext {
    /// Task ID, used for logging and tracing
    pub task_id: Option<String>,
    /// Current step index (starting from 0)
    pub skill_index: Option<usize>,
    /// Current step name
    pub skill_name: Option<String>,
    /// Extended data for future needs
    pub extra: HashMap<String, Value>,
}

impl SkillContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new context with task_id
    pub fn with_task_id(task_id: impl Into<String>) -> Self {
        Self {
            task_id: Some(task_id.into()),
            ..Default::default()
        }
    }

    /// Get task_id
    pub fn task_id(&self) -> Option<&str> {
        self.task_id.as_deref()
    }

    /// Get step_index
    pub fn skill_index(&self) -> Option<usize> {
        self.skill_index
    }

    /// Get step_name
    pub fn skill_name(&self) -> Option<&str> {
        self.skill_name.as_deref()
    }

    /// Get extra value by key
    pub fn get_extra(&self, key: &str) -> Option<&Value> {
        self.extra.get(key)
    }

    /// Set task_id
    pub fn set_task_id(&mut self, task_id: impl Into<String>) -> &mut Self {
        self.task_id = Some(task_id.into());
        self
    }

    /// Set skill_index
    pub fn set_skill_index(&mut self, skill_index: usize) -> &mut Self {
        self.skill_index = Some(skill_index);
        self
    }

    /// Set skill_name
    pub fn set_skill_name(&mut self, skill_name: impl Into<String>) -> &mut Self {
        self.skill_name = Some(skill_name.into());
        self
    }

    /// Insert extra value
    pub fn insert_extra(&mut self, key: impl Into<String>, value: Value) -> &mut Self {
        self.extra.insert(key.into(), value);
        self
    }

    /// Remove extra value
    pub fn remove_extra(&mut self, key: &str) -> Option<Value> {
        self.extra.remove(key)
    }

    /// Check if extra contains key
    pub fn has_extra(&self, key: &str) -> bool {
        self.extra.contains_key(key)
    }

    /// Builder: set task_id
    pub fn with_task_id_builder(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }

    /// Builder: set skill_index
    pub fn with_skill_index(mut self, skill_index: usize) -> Self {
        self.skill_index = Some(skill_index);
        self
    }

    /// Builder: set skill_name
    pub fn with_skill_name(mut self, skill_name: impl Into<String>) -> Self {
        self.skill_name = Some(skill_name.into());
        self
    }

    /// Builder: insert extra value
    pub fn with_extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }

    /// Builder: merge extra values from a HashMap
    pub fn with_extra_map(mut self, extra: HashMap<String, Value>) -> Self {
        self.extra.extend(extra);
        self
    }

    /// Builder: build the context
    pub fn build(self) -> Self {
        self
    }
}

/// Atomic skill progress callback
///
/// Implemented by the external layer and injected into skills.
pub trait SkillCallback: Send + Sync + Debug {
    /// log output
    fn on_log(&self, task_id: Option<String>, skill_index: Option<usize>, message: Option<String>);

    /// Progress update
    fn on_progress(
        &self,
        task_id: Option<String>,
        skill_index: Option<usize>,
        progress: Option<u32>,
        message: Option<String>,
    );

    /// Step started (optional, default implementation does nothing)
    fn on_start(
        &self,
        task_id: Option<String>,
        skill_index: Option<usize>,
        skill_name: Option<String>,
    ) {
    }

    /// Step completed (optional, default implementation does nothing)
    fn on_complete(
        &self,
        task_id: Option<String>,
        skill_index: Option<usize>,
        skill_name: Option<String>,
        output: Option<String>,
    ) {
    }

    /// Step failed (optional, default implementation does nothing)
    fn on_error(
        &self,
        task_id: Option<String>,
        skill_index: Option<usize>,
        skill_name: Option<String>,
        error: Option<String>,
    ) {
    }
}

/// Skill execution trait
///
/// This trait defines the contract for all skills in the system. Any type
/// implementing this trait can be executed by the skill router when an
/// LLM requests that action.
///
/// # Required Methods
///
/// - `name()`: Unique identifier for the skill
/// - `description()`: Brief one-line summary
/// - `execute()`: Core implementation that performs the skill's function
///
/// # Optional Methods
///
/// - `usage_hint()`: Detailed guidance for LLMs
/// - `parameters()`: Parameter definitions for validation
/// - `validate()`: Custom validation logic
/// - `get_metadata()`: Complete metadata for LLM consumption
///
/// # Example
///
/// ```rust,ignore
/// use anyhow::Result;
/// use std::collections::HashMap;
/// use serde_json::Value;
///
/// #[derive(Debug)]
/// struct EchoSkill;
///
/// #[async_trait::async_trait]
/// impl Skill for EchoSkill {
///     fn name(&self) -> &str { "echo" }
///     fn description(&self) -> &str { "Echo back the input message" }
///     
///     async fn execute(&self, params: &HashMap<String, Value>) -> Result<String> {
///         let message = params.get("message")
///             .and_then(|v| v.as_str())
///             .unwrap_or("");
///         Ok(message.to_string())
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait Skill: Send + Sync + Debug {
    /// Skill name (must match the `name` field in SKILL.md)
    ///
    /// This name is used as the `action` value in `SkillCall` to
    /// route execution to this skill. It should be unique across
    /// all registered skills.
    fn name(&self) -> &str;

    /// Brief description (one line)
    ///
    /// A short, human-readable summary of what this skill does.
    /// This is shown to the LLM to help it understand the skill's
    /// purpose at a glance.
    fn description(&self) -> &str;

    /// Detailed usage hint
    ///
    /// Provides comprehensive guidance on when and how to use this
    /// skill. This can include examples, edge cases, and context
    /// requirements.
    ///
    /// Default implementation returns a generic message.
    fn usage_hint(&self) -> &str {
        "No usage hint provided"
    }

    /// Parameter definitions
    ///
    /// Returns the list of parameters that this skill accepts.
    /// These definitions are used for validation and to generate
    /// LLM-facing documentation.
    ///
    /// Default implementation returns an empty vector.
    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    /// Example call (JSON format)
    ///
    /// Provides a concrete JSON example showing how to invoke this
    /// skill. This helps the LLM understand the expected structure.
    ///
    /// Default implementation returns an empty JSON object.
    fn example_call(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    /// Example output
    ///
    /// A string demonstrating the expected output format from this
    /// skill. This helps the LLM understand what to expect.
    ///
    /// Default implementation returns an empty string.
    fn example_output(&self) -> String {
        String::new()
    }

    /// Category (file, net, math, time, system)
    ///
    /// Returns the functional category of this skill for organizational
    /// purposes.
    ///
    /// Default implementation returns `"general"`.
    fn category(&self) -> SkillCategory {
        SkillCategory::Basic
    }

    /// Execute the skill with given parameters
    ///
    /// This is the main entry point for skill execution. Implementations
    /// should perform their core functionality here using the provided
    /// parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - A map of parameter names to JSON values
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The skill's output as a string, or an error
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails for any reason (invalid
    /// parameters, I/O errors, etc.).
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String>;

    /// Validate parameters before execution
    ///
    /// Performs validation on the provided parameters before execution
    /// begins. The default implementation checks required fields, type
    /// compatibility, and enum values based on the parameter definitions
    /// from `parameters()`.
    ///
    /// Override this method to implement custom validation logic.
    ///
    /// # Arguments
    ///
    /// * `parameters` - The parameters to validate
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if validation passes, Err otherwise
    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        // Default validation based on parameter definitions
        let param_defs = self.parameters();
        for def in param_defs {
            let param_name = &def.name;
            let has_value = parameters.contains_key(param_name);
            if def.required && !has_value {
                anyhow::bail!("Required parameter '{}' is missing", param_name);
            }
            if let Some(value) = parameters.get(param_name) {
                let type_matches = match def.param_type.as_str() {
                    "string" => value.is_string(),
                    "integer" => value.is_i64() || value.is_u64(),
                    "boolean" => value.is_boolean(),
                    "array" => value.is_array(),
                    "object" => value.is_object(),
                    _ => true, // Unknown type, skip validation
                };
                if !type_matches {
                    anyhow::bail!(
                        "Parameter '{}' expects type '{}' but got {:?}",
                        param_name,
                        def.param_type,
                        value
                    );
                }
                if let Some(enum_vals) = &def.enum_values {
                    if let Some(str_val) = value.as_str() {
                        if !enum_vals.contains(&str_val.to_string()) {
                            anyhow::bail!(
                                "Parameter '{}' value '{}' is not in allowed values: {:?}",
                                param_name,
                                str_val,
                                enum_vals
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Get full metadata for LLM
    ///
    /// Aggregates all skill information into a `SkillMetadata` structure
    /// suitable for serialization and inclusion in LLM prompts.
    ///
    /// # Returns
    ///
    /// * `SkillMetadata` - Complete metadata about this skill
    fn get_metadata(&self) -> SkillMetadata {
        SkillMetadata {
            name: self.name().to_string(),
            description: self.description().to_string(),
            usage_hint: self.usage_hint().to_string(),
            parameters: self.parameters(),
            example_call: self.example_call(),
            example_output: self.example_output(),
            category: self.category(),
        }
    }
}

/// Skill call instruction parsed from LLM response
///
/// Represents a request to execute a specific skill. This structure
/// is typically deserialized from JSON output generated by an LLM
/// after being prompted to call a skill.
///
/// # Example
///
/// ```rust
/// # use serde_json::json;
/// # use your_crate::SkillCall;
/// # use std::collections::HashMap;
/// let json_data = json!({
///     "action": "read_file",
///     "parameters": {
///         "path": "./config.json"
///     }
/// });
///
/// let call: SkillCall = serde_json::from_value(json_data).unwrap();
/// assert_eq!(call.action, "read_file");
/// assert_eq!(call.parameters.get("path").unwrap().as_str(), Some("./config.json"));
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SkillCall {
    /// The name of the skill to invoke
    ///
    /// This must match the `name()` of a registered skill for the
    /// call to be routed correctly.
    pub action: String,

    /// Parameters to pass to the skill
    ///
    /// A map from parameter names to their JSON values. The skill
    /// implementation is responsible for extracting and validating
    /// these values.
    #[serde(default)]
    pub parameters: HashMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Mock skill for testing
    #[derive(Debug)]
    struct TestSkill {
        name: String,
        description: String,
    }

    #[async_trait::async_trait]
    impl Skill for TestSkill {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        async fn execute(
            &self,
            params: &HashMap<String, Value>,
            callback: Option<&dyn SkillCallback>,
            context: Option<&SkillContext>,
        ) -> Result<String> {
            let result = params
                .get("input")
                .and_then(|v| v.as_str())
                .unwrap_or("no input");
            Ok(format!("Executed {} with: {}", self.name, result))
        }

        fn parameters(&self) -> Vec<SkillParameter> {
            vec![
                SkillParameter {
                    name: "input".to_string(),
                    param_type: "string".to_string(),
                    description: "Input string".to_string(),
                    required: true,
                    default: None,
                    example: Some(json!("test")),
                    enum_values: None,
                },
                SkillParameter {
                    name: "count".to_string(),
                    param_type: "integer".to_string(),
                    description: "Count value".to_string(),
                    required: false,
                    default: Some(json!(1)),
                    example: Some(json!(5)),
                    enum_values: None,
                },
            ]
        }

        fn usage_hint(&self) -> &str {
            "Use this skill to test functionality"
        }

        fn category(&self) -> SkillCategory {
            SkillCategory::Basic
        }
    }

    // Parameter validation skill for testing validation
    #[derive(Debug)]
    struct ValidatingSkill;

    #[async_trait::async_trait]
    impl Skill for ValidatingSkill {
        fn name(&self) -> &str {
            "validator"
        }

        fn description(&self) -> &str {
            "Validates parameters"
        }

        async fn execute(
            &self,
            params: &HashMap<String, Value>,
            callback: Option<&dyn SkillCallback>,
            context: Option<&SkillContext>,
        ) -> Result<String> {
            Ok(format!("Validated: {:?}", params))
        }

        fn parameters(&self) -> Vec<SkillParameter> {
            vec![
                SkillParameter {
                    name: "color".to_string(),
                    param_type: "string".to_string(),
                    description: "Color name".to_string(),
                    required: true,
                    default: None,
                    example: Some(json!("red")),
                    enum_values: Some(vec![
                        "red".to_string(),
                        "green".to_string(),
                        "blue".to_string(),
                    ]),
                },
                SkillParameter {
                    name: "value".to_string(),
                    param_type: "integer".to_string(),
                    description: "Numeric value".to_string(),
                    required: false,
                    default: Some(json!(0)),
                    example: Some(json!(42)),
                    enum_values: None,
                },
            ]
        }
    }

    #[tokio::test]
    async fn test_skill_metadata_creation() {
        let skill = TestSkill {
            name: "test_skill".to_string(),
            description: "A test skill".to_string(),
        };
        let metadata = skill.get_metadata();
        assert_eq!(metadata.name, "test_skill");
        assert_eq!(metadata.description, "A test skill");
        assert_eq!(metadata.usage_hint, "Use this skill to test functionality");
        assert_eq!(metadata.category, SkillCategory::Basic);
        assert_eq!(metadata.parameters.len(), 2);
        assert_eq!(metadata.parameters[0].name, "input");
        assert_eq!(metadata.parameters[0].required, true);
        assert_eq!(metadata.parameters[1].name, "count");
        assert_eq!(metadata.parameters[1].required, false);
    }

    #[tokio::test]
    async fn test_skill_execution_with_parameters() {
        let skill = TestSkill {
            name: "echo_skill".to_string(),
            description: "Echoes input".to_string(),
        };
        let mut params = HashMap::new();
        params.insert("input".to_string(), json!("Hello, World!"));
        let result = skill.execute(&params, None, None).await.unwrap();
        assert_eq!(result, "Executed echo_skill with: Hello, World!");
    }

    #[tokio::test]
    async fn test_skill_validation() {
        let skill = ValidatingSkill;
        let mut valid_params = HashMap::new();
        valid_params.insert("color".to_string(), json!("red"));
        valid_params.insert("value".to_string(), json!(42));
        let result = skill.validate(&valid_params);
        assert!(result.is_ok());
        let mut missing_required = HashMap::new();
        missing_required.insert("value".to_string(), json!(42));
        let result = skill.validate(&missing_required);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Required parameter 'color'")
        );
        let mut invalid_enum = HashMap::new();
        invalid_enum.insert("color".to_string(), json!("yellow"));
        let result = skill.validate(&invalid_enum);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not in allowed values")
        );
        let mut wrong_type = HashMap::new();
        wrong_type.insert("color".to_string(), json!("red"));
        wrong_type.insert("value".to_string(), json!("not an integer"));
        let result = skill.validate(&wrong_type);
        assert!(result.is_err());
    }

    #[test]
    fn test_skill_call_deserialization() {
        let json_data = json!({
            "action": "read_file",
            "parameters": {
                "path": "./config.json",
                "encoding": "utf-8"
            }
        });
        let call: SkillCall = serde_json::from_value(json_data).unwrap();
        assert_eq!(call.action, "read_file");
        assert_eq!(call.parameters.len(), 2);
        assert_eq!(
            call.parameters.get("path").unwrap().as_str(),
            Some("./config.json")
        );
        assert_eq!(
            call.parameters.get("encoding").unwrap().as_str(),
            Some("utf-8")
        );
    }

    #[test]
    fn test_skill_call_without_parameters() {
        let json_data = json!({
            "action": "list_skills"
        });
        let call: SkillCall = serde_json::from_value(json_data).unwrap();
        assert_eq!(call.action, "list_skills");
        assert!(call.parameters.is_empty());
    }

    #[test]
    fn test_skill_parameter_serialization() {
        let param = SkillParameter {
            name: "timeout".to_string(),
            param_type: "integer".to_string(),
            description: "Timeout in seconds".to_string(),
            required: true,
            default: Some(json!(30)),
            example: Some(json!(60)),
            enum_values: None,
        };
        let serialized = serde_json::to_string(&param).unwrap();
        let deserialized: SkillParameter = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.name, param.name);
        assert_eq!(deserialized.param_type, param.param_type);
        assert_eq!(deserialized.required, param.required);
        assert_eq!(deserialized.default, param.default);
    }

    #[test]
    fn test_skill_parameter_with_enum_values() {
        let param = SkillParameter {
            name: "mode".to_string(),
            param_type: "string".to_string(),
            description: "Operation mode".to_string(),
            required: true,
            default: None,
            example: Some(json!("fast")),
            enum_values: Some(vec![
                "fast".to_string(),
                "slow".to_string(),
                "balanced".to_string(),
            ]),
        };
        assert_eq!(param.enum_values.as_ref().unwrap().len(), 3);
        assert!(
            param
                .enum_values
                .as_ref()
                .unwrap()
                .contains(&"fast".to_string())
        );
    }

    #[test]
    fn test_context_new() {
        let ctx = SkillContext::new();
        assert!(ctx.task_id().is_none());
        assert!(ctx.skill_index().is_none());
        assert!(ctx.skill_name().is_none());
        assert!(ctx.extra.is_empty());
    }

    #[test]
    fn test_context_with_task_id() {
        let ctx = SkillContext::with_task_id("task-123");
        assert_eq!(ctx.task_id(), Some("task-123"));
    }

    #[test]
    fn test_context_getters_setters() {
        let mut ctx = SkillContext::new();
        ctx.set_task_id("task-456")
            .set_skill_index(3)
            .set_skill_name("download_file");

        assert_eq!(ctx.task_id(), Some("task-456"));
        assert_eq!(ctx.skill_index(), Some(3));
        assert_eq!(ctx.skill_name(), Some("download_file"));
    }

    #[test]
    fn test_context_extra_operations() {
        let mut ctx = SkillContext::new();
        ctx.insert_extra("url", json!("https://example.com"))
            .insert_extra("timeout", json!(30));

        assert!(ctx.has_extra("url"));
        assert_eq!(ctx.get_extra("url"), Some(&json!("https://example.com")));
        assert_eq!(ctx.get_extra("timeout"), Some(&json!(30)));

        let removed = ctx.remove_extra("timeout");
        assert_eq!(removed, Some(json!(30)));
        assert!(!ctx.has_extra("timeout"));
    }

    #[test]
    fn test_context_builder() {
        let ctx = SkillContext::new()
            .with_task_id_builder("task-789")
            .with_skill_index(5)
            .with_skill_name("process_data")
            .with_extra("retry_count", json!(3))
            .build();
        assert_eq!(ctx.task_id(), Some("task-789"));
        assert_eq!(ctx.skill_index(), Some(5));
        assert_eq!(ctx.skill_name(), Some("process_data"));
        assert_eq!(ctx.get_extra("retry_count"), Some(&json!(3)));
    }
}
