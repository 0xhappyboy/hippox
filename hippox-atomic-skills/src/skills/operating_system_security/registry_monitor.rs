//! Registry monitor skill (Windows)

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCallback, SkillCategory, SkillContext, types::{Skill, SkillParameter}
};

#[cfg(target_os = "windows")]
use crate::operating_system_security::common::monitor_registry_key;

#[derive(Debug)]
pub struct RegistryMonitorSkill;

#[async_trait::async_trait]
impl Skill for RegistryMonitorSkill {
    fn name(&self) -> &str {
        "security_registry_monitor"
    }

    fn description(&self) -> &str {
        "Monitor Windows registry keys for security issues"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to audit Windows registry keys for security issues like startup persistence and service configurations"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Registry key path to monitor (default: HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run)".to_string(),
                required: false,
                default: Some(Value::String("HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string())),
                example: Some(Value::String("HKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Services".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "security_registry_monitor",
            "parameters": {
                "key": "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run"
            }
        })
    }

    fn example_output(&self) -> String {
        "Registry Monitor Results:\n\nKey: HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\nName: startup\nValue: C:\\Program Files\\App\\app.exe\nValue Type: REG_SZ\nSecurity Issues:\n  - Startup registry key - potential persistence".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::OperatingSystemSecurity
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        #[cfg(not(target_os = "windows"))]
        {
            return Ok("Registry monitor is only supported on Windows systems".to_string());
        }

        #[cfg(target_os = "windows")]
        {
            let key = parameters
                .get("key")
                .and_then(|v| v.as_str())
                .unwrap_or("HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run");

            let info = monitor_registry_key(key);

            let mut output = String::new();
            output.push_str(&format!(
                "Registry Monitor Results:\n\nKey: {}\n",
                info.path
            ));
            output.push_str(&format!("Name: {}\n", info.name));
            output.push_str(&format!("Value: {}\n", info.value));
            output.push_str(&format!("Value Type: {}\n", info.value_type));

            if !info.security_issues.is_empty() {
                output.push_str("\nSecurity Issues:\n");
                for issue in &info.security_issues {
                    output.push_str(&format!("  - {}\n", issue));
                }
            } else {
                output.push_str("\nNo security issues found.");
            }

            Ok(output)
        }
    }
}
