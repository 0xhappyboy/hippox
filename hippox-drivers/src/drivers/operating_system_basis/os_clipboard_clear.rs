//! Clipboard clear driver
//!
//! This driver provides functionality to clear the system clipboard content.
use super::os_clipboard_set::ClipboardSetDriver;
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for clearing clipboard content
#[derive(Debug)]
pub struct ClipboardClearDriver;
#[async_trait::async_trait]
impl Driver for ClipboardClearDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "os_clipboard_clear"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Clear system clipboard content"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to clear clipboard content"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "os_clipboard_clear",
            "parameters": {}
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Clipboard cleared".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemBasis;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing os_clipboard_clear driver");
        info!("Clearing clipboard content");
        return ClipboardSetDriver
            .execute(
                &{
                    let mut params = HashMap::new();
                    params.insert("content".to_string(), Value::String(String::new()));
                    params
                },
                callback,
                context,
            )
            .await;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_clipboard_clear_metadata() {
        let driver = ClipboardClearDriver;
        assert_eq!(driver.name(), "os_clipboard_clear");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
