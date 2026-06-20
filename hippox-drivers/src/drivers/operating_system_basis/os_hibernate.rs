//! OS hibernate driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
#[derive(Debug)]
pub struct OsHibernateDriver;
#[async_trait::async_trait]
impl Driver for OsHibernateDriver {
    fn name(&self) -> &str {
        "os_hibernate"
    }
    fn description(&self) -> &str {
        "Hibernate the system (suspend to disk)"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to save power while preserving system state to disk"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_hibernate"
        })
    }
    fn example_output(&self) -> String {
        "System is hibernating".to_string()
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemBasis
    }
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> Result<String> {
        #[cfg(target_os = "windows")]
        {
            use crate::exec_async;
            exec_async("shutdown", &["/h"], None).await?;
        }
        #[cfg(target_os = "linux")]
        {
            exec_async("systemctl", &["hibernate"], None).await?;
        }
        #[cfg(target_os = "macos")]
        {
            exec_async("pmset", &["sleepnow"], None).await?;
        }
        Ok("System is hibernating".to_string())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_hibernate_metadata() {
        let driver = OsHibernateDriver;
        assert_eq!(driver.name(), "os_hibernate");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
