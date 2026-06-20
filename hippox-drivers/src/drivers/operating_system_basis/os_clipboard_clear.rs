//! Clipboard clear driver
use super::os_clipboard_set::ClipboardSetDriver;
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
#[derive(Debug)]
pub struct ClipboardClearDriver;
#[async_trait::async_trait]
impl Driver for ClipboardClearDriver {

    fn name(&self) -> &str {
        "os_clipboard_clear"
    }

    fn description(&self) -> &str {
        "Clear system clipboard content"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to clear clipboard content"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "os_clipboard_clear",
            "parameters": {}
        })
    }

    fn example_output(&self) -> String {
        "Clipboard cleared".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemBasis
    }

    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        ClipboardSetDriver
            .execute(
                &{
                    let mut params = HashMap::new();
                    params.insert("content".to_string(), Value::String(String::new()));
                    params
                },
                callback,
                context,
            )
            .await
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
