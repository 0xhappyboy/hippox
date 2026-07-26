//! Browser execute JavaScript skill
//!
//! This driver provides functionality to execute JavaScript code
//! in the context of the current page.
use super::shared::*;
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for executing JavaScript in the current page
#[derive(Debug)]
pub struct HaveHeadBrowserExecuteJsDriver;
#[async_trait::async_trait]
impl Driver for HaveHeadBrowserExecuteJsDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "have_head_browser_execute_js"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Execute JavaScript code in the current page"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to interact with page JavaScript, modify DOM, or extract complex data"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "code".to_string(),
                param_type: "string".to_string(),
                description: "JavaScript code to execute".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("document.title".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "return_value".to_string(),
                param_type: "boolean".to_string(),
                description: "Return the result of the code (default: true)".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "have_head_browser_execute_js",
            "parameters": {
                "code": "return document.querySelectorAll('a').length;"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Result: 42".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::HaveHeadBrowser;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing have_head_browser_execute_js driver");
        let code = parameters.get("code").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing required parameter: code");
            return crate::DriverError::missing_parameter("code");
        })?;
        let return_value = parameters.get("return_value").and_then(|v| v.as_bool()).unwrap_or(true);
        debug!("Executing JavaScript (return_value: {})", return_value);
        let tab = get_current_tab().map_err(|e| {
            debug!("Failed to get current tab: {}", e);
            return crate::DriverError::execution(format!("Failed to get current tab: {}", e));
        })?;
        if return_value {
            let result = tab.evaluate(code, false).map_err(|e| {
                warn!("Failed to execute JS: {}", e);
                return crate::DriverError::execution(format!("Failed to execute JS: {}", e));
            })?;
            let value = result.value.map(|v| v.to_string()).unwrap_or_else(|| "null".to_string());
            info!("JavaScript executed, result: {}", value);
            return Ok(format!("Result: {}", value));
        } else {
            tab.evaluate(code, false).map_err(|e| {
                warn!("Failed to execute JS: {}", e);
                return crate::DriverError::execution(format!("Failed to execute JS: {}", e));
            })?;
            info!("JavaScript executed successfully");
            return Ok("JavaScript executed successfully".to_string());
        }
    }
}
