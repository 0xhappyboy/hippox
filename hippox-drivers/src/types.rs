use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::RwLock;

use crate::{DriverCategory, DriverSignal};

/// Driver parameter definition
///
/// Defines a single parameter that can be passed to a driver. This includes
/// type information, validation constraints, and documentation to help LLMs
/// understand how to use the parameter correctly.
///
/// # Examples
///
/// ```rust
/// # use serde_json::json;
/// # use your_crate::DriverParameter;
/// let param = DriverParameter {
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
pub struct DriverParameter {
    /// Parameter name (used in JSON call)
    ///
    /// This name must be used as the key when providing the parameter
    /// in a driver call's parameters map.
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
    /// If `true`, the driver will fail validation if this parameter
    /// is missing from the call. If `false`, the driver should use
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

/// Complete Driver metadata for LLM
///
/// Contains all information about a Driver that is needed by the LLM
/// to understand when and how to invoke the Driver. This metadata is
/// typically serialized to JSON and included in the system prompt
/// or tool definitions.
///
/// # Example
///
/// ```rust
/// # use serde_json::json;
/// # use your_crate::{DriverMetadata, DriverParameter};
/// let metadata = DriverMetadata {
///     name: "read_file".to_string(),
///     description: "Read contents of a file".to_string(),
///     usage_hint: "Use when you need to examine file contents".to_string(),
///     parameters: vec![
///         DriverParameter {
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
pub struct DriverMetadata {
    /// Driver name (used as action in DriverCall)
    ///
    /// This name must match the `action` field in a `DriverCall` to
    /// invoke this Driver.
    pub name: String,

    /// Brief description of what this Driver does
    ///
    /// One-sentence summary that helps the LLM quickly identify
    /// whether this Driver is appropriate for the current task.
    pub description: String,

    /// Detailed explanation of when to use this Driver
    ///
    /// Provides guidance on the specific scenarios where this Driver
    /// should be invoked, including any prerequisites or context
    /// requirements.
    pub usage_hint: String,

    /// Parameter definitions
    ///
    /// Complete list of parameters that this Driver accepts, including
    /// type information, validation rules, and documentation.
    pub parameters: Vec<DriverParameter>,

    /// Example of how to call this Driver (JSON format)
    ///
    /// A concrete example showing the expected JSON structure for
    /// invoking this Driver, including parameter names and example values.
    pub example_call: serde_json::Value,

    /// Example output format
    ///
    /// A string describing or demonstrating the expected output format
    /// so the LLM knows what to expect when the Driver returns.
    pub example_output: String,

    /// Category for grouping (file, net, math, time, system)
    ///
    /// Used to organize Drivers by functional area. Common categories:
    /// - `"file"`: File system operations
    /// - `"net"`: Network/HTTP operations
    /// - `"math"`: Mathematical computations
    /// - `"time"`: Time/date operations
    /// - `"system"`: System-level operations
    pub category: DriverCategory,
}

/// Atomic Driver execution context
///
/// Passed from the external layer to atomic Drivers. Drivers are read-only consumers
/// and do not know the implementation details of the upper layer.
#[derive(Debug, Clone, Default)]
pub struct DriverContext {
    /// Task ID, used for logging and tracing
    pub task_id: Option<String>,
    /// Current step index (starting from 0)
    pub driver_index: Option<usize>,
    /// Current step name
    pub driver_name: Option<String>,
    /// Extended data for future needs
    pub extra: HashMap<String, Value>,
    pub signal_bus: Option<&'static RwLock<HashMap<String, HashMap<usize, DriverSignal>>>>,
}

impl DriverContext {
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
    pub fn driver_index(&self) -> Option<usize> {
        self.driver_index
    }

    /// Get step_name
    pub fn driver_name(&self) -> Option<&str> {
        self.driver_name.as_deref()
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

    /// Set driver_index
    pub fn set_driver_index(&mut self, driver_index: usize) -> &mut Self {
        self.driver_index = Some(driver_index);
        self
    }

    /// Set driver_name
    pub fn set_driver_name(&mut self, driver_name: impl Into<String>) -> &mut Self {
        self.driver_name = Some(driver_name.into());
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

    /// Builder: set driver_index
    pub fn with_driver_index(mut self, driver_index: usize) -> Self {
        self.driver_index = Some(driver_index);
        self
    }

    /// Builder: set driver_name
    pub fn with_driver_name(mut self, driver_name: impl Into<String>) -> Self {
        self.driver_name = Some(driver_name.into());
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

/// Atomic driver progress callback
///
/// Implemented by the external layer and injected into drivers.
pub trait DriverCallback: Send + Sync + Debug {
    /// log output
    fn on_log(&self, task_id: Option<String>, driver_index: Option<usize>, message: Option<String>);

    /// Callback function for output from inside the driver to the outside.
    fn on_output(
        &self,
        task_id: Option<String>,
        driver_index: Option<usize>,
        driver_name: Option<String>,
        output: Option<String>,
    ) {
    }

    /// Progress update
    fn on_progress(
        &self,
        task_id: Option<String>,
        driver_index: Option<usize>,
        progress: Option<u32>,
        message: Option<String>,
    );

    /// Step started (optional, default implementation does nothing)
    fn on_start(
        &self,
        task_id: Option<String>,
        driver_index: Option<usize>,
        driver_name: Option<String>,
    ) {
    }

    /// Step completed (optional, default implementation does nothing)
    fn on_complete(
        &self,
        task_id: Option<String>,
        driver_index: Option<usize>,
        driver_name: Option<String>,
        output: Option<String>,
    ) {
    }

    /// Step failed (optional, default implementation does nothing)
    fn on_error(
        &self,
        task_id: Option<String>,
        driver_index: Option<usize>,
        driver_name: Option<String>,
        error: Option<String>,
    ) {
    }
}

/// Skill execution trait
///
/// This trait defines the contract for all drivers in the system. Any type
/// implementing this trait can be executed by the driver router when an
/// LLM requests that action.
///
/// # Required Methods
///
/// - `name()`: Unique identifier for the driver
/// - `description()`: Brief one-line summary
/// - `execute()`: Core implementation that performs the driver's function
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
pub trait Driver: Send + Sync + Debug {
    /// Driver name (must match the `name` field in Driver.md)
    ///
    /// This name is used as the `action` value in `DriverCall` to
    /// route execution to this Driver. It should be unique across
    /// all registered Drivers.
    fn name(&self) -> &str;

    /// Brief description (one line)
    ///
    /// A short, human-readable summary of what this Driver does.
    /// This is shown to the LLM to help it understand the Driver's
    /// purpose at a glance.
    fn description(&self) -> &str;

    /// Detailed usage hint
    ///
    /// Provides comprehensive guidance on when and how to use this
    /// Driver. This can include examples, edge cases, and context
    /// requirements.
    ///
    /// Default implementation returns a generic message.
    fn usage_hint(&self) -> &str {
        "No usage hint provided"
    }

    /// Parameter definitions
    ///
    /// Returns the list of parameters that this Driver accepts.
    /// These definitions are used for validation and to generate
    /// LLM-facing documentation.
    ///
    /// Default implementation returns an empty vector.
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    /// Example call (JSON format)
    ///
    /// Provides a concrete JSON example showing how to invoke this
    /// Driver. This helps the LLM understand the expected structure.
    ///
    /// Default implementation returns an empty JSON object.
    fn example_call(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    /// Example output
    ///
    /// A string demonstrating the expected output format from this
    /// Driver. This helps the LLM understand what to expect.
    ///
    /// Default implementation returns an empty string.
    fn example_output(&self) -> String {
        String::new()
    }

    /// Category (file, net, math, time, system)
    ///
    /// Returns the functional category of this Driver for organizational
    /// purposes.
    ///
    /// Default implementation returns `"general"`.
    fn category(&self) -> DriverCategory {
        DriverCategory::Basic
    }

    /// Execute the Driver with given parameters
    ///
    /// This is the main entry point for Driver execution. Implementations
    /// should perform their core functionality here using the provided
    /// parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - A map of parameter names to JSON values
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The Driver's output as a string, or an error
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails for any reason (invalid
    /// parameters, I/O errors, etc.).
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
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
    /// Aggregates all Driver information into a `DriverMetadata` structure
    /// suitable for serialization and inclusion in LLM prompts.
    ///
    /// # Returns
    ///
    /// * `DriverMetadata` - Complete metadata about this Driver
    fn get_metadata(&self) -> DriverMetadata {
        DriverMetadata {
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
/// Represents a request to execute a specific driver. This structure
/// is typically deserialized from JSON output generated by an LLM
/// after being prompted to call a driver.
///
/// # Example
///
/// ```rust
/// # use serde_json::json;
/// # use your_crate::DriverCall;
/// # use std::collections::HashMap;
/// let json_data = json!({
///     "action": "read_file",
///     "parameters": {
///         "path": "./config.json"
///     }
/// });
///
/// let call: DriverCall = serde_json::from_value(json_data).unwrap();
/// assert_eq!(call.action, "read_file");
/// assert_eq!(call.parameters.get("path").unwrap().as_str(), Some("./config.json"));
/// ```
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DriverCall {
    /// The name of the driver to invoke
    ///
    /// This must match the `name()` of a registered driver for the
    /// call to be routed correctly.
    pub action: String,

    /// Parameters to pass to the driver
    ///
    /// A map from parameter names to their JSON values. The driver
    /// implementation is responsible for extracting and validating
    /// these values.
    #[serde(default)]
    pub parameters: HashMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Mock Driver for testing
    #[derive(Debug)]
    struct TestDriver {
        name: String,
        description: String,
    }

    #[async_trait::async_trait]
    impl Driver for TestDriver {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        async fn execute(
            &self,
            params: &HashMap<String, Value>,
            callback: Option<&dyn DriverCallback>,
            context: Option<&DriverContext>,
        ) -> Result<String> {
            let result = params
                .get("input")
                .and_then(|v| v.as_str())
                .unwrap_or("no input");
            Ok(format!("Executed {} with: {}", self.name, result))
        }

        fn parameters(&self) -> Vec<DriverParameter> {
            vec![
                DriverParameter {
                    name: "input".to_string(),
                    param_type: "string".to_string(),
                    description: "Input string".to_string(),
                    required: true,
                    default: None,
                    example: Some(json!("test")),
                    enum_values: None,
                },
                DriverParameter {
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
            "Use this Driver to test functionality"
        }

        fn category(&self) -> DriverCategory {
            DriverCategory::Basic
        }
    }

    // Parameter validation Driver for testing validation
    #[derive(Debug)]
    struct ValidatingDriver;

    #[async_trait::async_trait]
    impl Driver for ValidatingDriver {
        fn name(&self) -> &str {
            "validator"
        }

        fn description(&self) -> &str {
            "Validates parameters"
        }

        async fn execute(
            &self,
            params: &HashMap<String, Value>,
            callback: Option<&dyn DriverCallback>,
            context: Option<&DriverContext>,
        ) -> Result<String> {
            Ok(format!("Validated: {:?}", params))
        }

        fn parameters(&self) -> Vec<DriverParameter> {
            vec![
                DriverParameter {
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
                DriverParameter {
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
    async fn test_driver_metadata_creation() {
        let driver = TestDriver {
            name: "test_driver".to_string(),
            description: "A test driver".to_string(),
        };
        let metadata = driver.get_metadata();
        assert_eq!(metadata.name, "test_Driver");
        assert_eq!(metadata.description, "A test Driver");
        assert_eq!(metadata.usage_hint, "Use this Driver to test functionality");
        assert_eq!(metadata.category, DriverCategory::Basic);
        assert_eq!(metadata.parameters.len(), 2);
        assert_eq!(metadata.parameters[0].name, "input");
        assert_eq!(metadata.parameters[0].required, true);
        assert_eq!(metadata.parameters[1].name, "count");
        assert_eq!(metadata.parameters[1].required, false);
    }

    #[tokio::test]
    async fn test_driver_execution_with_parameters() {
        let Driver = TestDriver {
            name: "echo_Driver".to_string(),
            description: "Echoes input".to_string(),
        };
        let mut params = HashMap::new();
        params.insert("input".to_string(), json!("Hello, World!"));
        let result = Driver.execute(&params, None, None).await.unwrap();
        assert_eq!(result, "Executed echo_Driver with: Hello, World!");
    }

    #[tokio::test]
    async fn test_driver_validation() {
        let Driver = ValidatingDriver;
        let mut valid_params = HashMap::new();
        valid_params.insert("color".to_string(), json!("red"));
        valid_params.insert("value".to_string(), json!(42));
        let result = Driver.validate(&valid_params);
        assert!(result.is_ok());
        let mut missing_required = HashMap::new();
        missing_required.insert("value".to_string(), json!(42));
        let result = Driver.validate(&missing_required);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Required parameter 'color'")
        );
        let mut invalid_enum = HashMap::new();
        invalid_enum.insert("color".to_string(), json!("yellow"));
        let result = Driver.validate(&invalid_enum);
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
        let result = Driver.validate(&wrong_type);
        assert!(result.is_err());
    }

    #[test]
    fn test_driver_call_deserialization() {
        let json_data = json!({
            "action": "read_file",
            "parameters": {
                "path": "./config.json",
                "encoding": "utf-8"
            }
        });
        let call: DriverCall = serde_json::from_value(json_data).unwrap();
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
    fn test_driver_call_without_parameters() {
        let json_data = json!({
            "action": "list_Drivers"
        });
        let call: DriverCall = serde_json::from_value(json_data).unwrap();
        assert_eq!(call.action, "list_Drivers");
        assert!(call.parameters.is_empty());
    }

    #[test]
    fn test_driver_parameter_serialization() {
        let param = DriverParameter {
            name: "timeout".to_string(),
            param_type: "integer".to_string(),
            description: "Timeout in seconds".to_string(),
            required: true,
            default: Some(json!(30)),
            example: Some(json!(60)),
            enum_values: None,
        };
        let serialized = serde_json::to_string(&param).unwrap();
        let deserialized: DriverParameter = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.name, param.name);
        assert_eq!(deserialized.param_type, param.param_type);
        assert_eq!(deserialized.required, param.required);
        assert_eq!(deserialized.default, param.default);
    }

    #[test]
    fn test_driver_parameter_with_enum_values() {
        let param = DriverParameter {
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
        let ctx = DriverContext::new();
        assert!(ctx.task_id().is_none());
        assert!(ctx.driver_index().is_none());
        assert!(ctx.driver_name().is_none());
        assert!(ctx.extra.is_empty());
    }

    #[test]
    fn test_context_with_task_id() {
        let ctx = DriverContext::with_task_id("task-123");
        assert_eq!(ctx.task_id(), Some("task-123"));
    }

    #[test]
    fn test_context_getters_setters() {
        let mut ctx = DriverContext::new();
        ctx.set_task_id("task-456")
            .set_driver_index(3)
            .set_driver_name("download_file");
        assert_eq!(ctx.task_id(), Some("task-456"));
        assert_eq!(ctx.driver_index(), Some(3));
        assert_eq!(ctx.driver_name(), Some("download_file"));
    }

    #[test]
    fn test_context_extra_operations() {
        let mut ctx = DriverContext::new();
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
        let ctx = DriverContext::new()
            .with_task_id_builder("task-789")
            .with_driver_index(5)
            .with_driver_name("process_data")
            .with_extra("retry_count", json!(3))
            .build();
        assert_eq!(ctx.task_id(), Some("task-789"));
        assert_eq!(ctx.driver_index(), Some(5));
        assert_eq!(ctx.driver_name(), Some("process_data"));
        assert_eq!(ctx.get_extra("retry_count"), Some(&json!(3)));
    }
}
