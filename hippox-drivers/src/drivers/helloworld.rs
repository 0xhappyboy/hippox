//! Hello World driver module
//!
//! This module provides a simple greeting driver for testing and demonstration purposes.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// A simple greeting driver for testing purposes
#[derive(Debug)]
pub struct HelloWorldDriver;
#[async_trait::async_trait]
impl Driver for HelloWorldDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "helloworld";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Greet a user by name";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this driver when the user asks to be greeted or when you need to introduce yourself";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "name".to_string(),
            param_type: "string".to_string(),
            description: "The name of the person to greet".to_string(),
            required: false,
            default: Some(Value::String("World".to_string())),
            example: Some(Value::String("Alice".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "helloworld",
            "parameters": {
                "name": "Alice"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Hello, Alice!".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Basic;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing helloworld driver");
        let name = parameters.get("name").and_then(|v| v.as_str()).unwrap_or("World");
        let result = format!("Hello, {}!", name);
        info!("{}", result);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, _parameters: &HashMap<String, Value>) -> DriverResult<()> {
        return Ok(());
    }
}
