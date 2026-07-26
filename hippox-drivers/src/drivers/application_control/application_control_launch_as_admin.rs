//! Application launch as admin driver
//!
//! This driver provides functionality to launch an application with
//! administrator privileges (may trigger UAC prompt on Windows).
use super::common::launch_as_admin;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for launching applications with administrator privileges
#[derive(Debug)]
pub struct ApplicationControlLaunchAsAdminDriver;
#[async_trait::async_trait]
impl Driver for ApplicationControlLaunchAsAdminDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "application_control_launch_as_admin"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Launch an application with administrator privileges"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to run an application as administrator. May trigger UAC prompt."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the application executable".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("cmd.exe".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "args".to_string(),
                param_type: "array".to_string(),
                description: "Command-line arguments".to_string(),
                required: false,
                default: Some(Value::Array(vec![])),
                example: Some(json!(["/c", "echo hello"])),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        Ok(json!({
            "action": "application_control_launch_as_admin",
            "parameters": {
                "path": "cmd.exe",
                "args": ["/c", "echo hello"]
            }
        }))
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        "Application launched as admin with PID: 12345".to_string()
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        DriverCategory::Application
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing application_control_launch_as_admin driver");
        // Extract the application path parameter
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'path' parameter");
            DriverError::missing_parameter("path")
        })?;
        // Extract the arguments parameter (optional)
        let args = parameters
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
            .unwrap_or_default();
        debug!("Launching application as admin: {} {:?}", path, args);
        // Launch the application with admin privileges
        let pid = launch_as_admin(path, &args).map_err(|e| DriverError::execution(format!("Failed to launch as admin: {}", e)))?;
        info!("Application launched as admin: {} (PID: {})", path, pid);
        Ok(format!("Application launched as admin with PID: {}", pid))
    }
}
