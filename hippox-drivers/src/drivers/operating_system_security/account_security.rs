//! Account security check driver
//!
//! This driver provides functionality to check account security including
//! password policy, locked accounts, and empty passwords.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_security::common::check_account_security,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for checking account security
#[derive(Debug)]
pub struct AccountSecurityDriver;
#[async_trait::async_trait]
impl Driver for AccountSecurityDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_account_check"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Check account security including password policy, locked accounts, and empty passwords"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to audit user accounts for security issues like weak passwords, locked accounts, and system accounts"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "Username to check (default: current user)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("root".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "check_all".to_string(),
                param_type: "boolean".to_string(),
                description: "Check all system users (default: false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "security_account_check",
            "parameters": {
                "username": "root"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Account Security Check Results:\n\nUsername: root\nUID: 0\nHome: /root\nShell: /bin/bash\nRoot account: Yes\nSystem account: No\nIssues:\n  - Root account detected - consider using sudo instead".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemSecurity;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing security_account_check driver");
        let username = parameters.get("username").and_then(|v| v.as_str());
        let check_all = parameters.get("check_all").and_then(|v| v.as_bool()).unwrap_or(false);
        info!("Checking account security: username={:?}, check_all={}", username, check_all);
        let mut output = String::new();
        output.push_str("Account Security Check Results:\n\n");
        if check_all {
            debug!("Checking all system accounts");
            use sysinfo::Users;
            let users = Users::new_with_refreshed_list();
            let mut results = Vec::new();
            for user in users.iter() {
                let username_str = user.name().to_string();
                let result = check_account_security(&username_str);
                results.push(result);
            }
            let issues: Vec<_> = results.iter().filter(|r| !r.issues.is_empty()).collect();
            let root_users: Vec<_> = results.iter().filter(|r| r.is_root).collect();
            if !root_users.is_empty() {
                output.push_str(&format!("Root accounts: {}\n", root_users.len()));
                for user in root_users {
                    output.push_str(&format!("  - {}\n", user.username));
                }
                output.push_str("\n");
            }
            if issues.is_empty() {
                output.push_str("No account security issues found.");
                info!("No account security issues found");
            } else {
                output.push_str("Account Security Issues:\n");
                let issues_size = issues.len();
                for result in issues {
                    output.push_str(&format!("  {}:\n", result.username));
                    for issue in &result.issues {
                        output.push_str(&format!("    - {}\n", issue));
                    }
                }
                info!("Found {} accounts with security issues", issues_size);
            }
        } else {
            let target = match username {
                Some(u) => u.to_string(),
                None => {
                    #[cfg(unix)]
                    {
                        std::env::var("USER").unwrap_or_else(|_| "unknown".to_string())
                    }
                    #[cfg(windows)]
                    {
                        std::env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string())
                    }
                }
            };
            debug!("Checking single account: {}", target);
            let result = check_account_security(&target);
            output.push_str(&format!("Username: {}\n", result.username));
            output.push_str(&format!("UID: {}\n", result.uid));
            output.push_str(&format!("GID: {}\n", result.gid));
            output.push_str(&format!("Home: {}\n", result.home_dir));
            output.push_str(&format!("Shell: {}\n", result.shell));
            output.push_str(&format!("Root account: {}\n", result.is_root));
            output.push_str(&format!("System account: {}\n", result.is_system));
            output.push_str(&format!("Account locked: {}\n", result.account_locked));
            if !result.issues.is_empty() {
                output.push_str("\nIssues:\n");
                for issue in &result.issues {
                    output.push_str(&format!("  - {}\n", issue));
                }
                info!("Account '{}' has {} issues", target, result.issues.len());
            } else {
                output.push_str("\nNo security issues found.");
                info!("Account '{}' has no security issues", target);
            }
        }
        return Ok(output);
    }
}
