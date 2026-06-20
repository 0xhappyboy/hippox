//! OS get user driver
use crate::{
    DriverCallback, DriverCategory, DriverContext,
    types::{Driver, DriverParameter},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use sysinfo::Users;
#[derive(Debug)]
pub struct OsGetUserDriver;
#[async_trait::async_trait]
impl Driver for OsGetUserDriver {
    fn name(&self) -> &str {
        "os_get_user"
    }
    fn description(&self) -> &str {
        "Get current user information"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill to get the current username, home directory, and user ID"
    }
    fn parameters(&self) -> Vec<DriverParameter> {
        vec![]
    }
    fn example_call(&self) -> Value {
        json!({
            "action": "os_get_user"
        })
    }
    fn example_output(&self) -> String {
        "Username: john\nUID: 1000\nGroups: sudo, docker".to_string()
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
        let users = Users::new_with_refreshed_list();
        let current_user = users.iter().next();
        if let Some(user) = current_user {
            let groups: Vec<String> = user.groups().iter().map(|g| g.name().to_string()).collect();
            Ok(format!(
                "Username: {}\nUID: {}\nGroups: {}",
                user.name().to_string(),
                user.id().to_string(),
                groups.join(", ")
            ))
        } else {
            Ok("User information could not be found.".to_string())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_os_get_user_metadata() {
        let driver = OsGetUserDriver;
        assert_eq!(driver.name(), "os_get_user");
        assert_eq!(driver.category(), DriverCategory::OperatingSystemBasis);
    }
}
