// application_control/application_control_uninstall.rs
//! Application uninstall skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::types::{Skill, SkillParameter};

#[derive(Debug)]
pub struct ApplicationControlUninstallSkill;

#[async_trait::async_trait]
impl Skill for ApplicationControlUninstallSkill {
    fn name(&self) -> &str {
        "application_control_uninstall"
    }

    fn description(&self) -> &str {
        "Uninstall an application using the system package manager"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to remove software packages. On Windows, uses winget. On Linux, uses apt/yum. On macOS, uses brew."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "package".to_string(),
            param_type: "string".to_string(),
            description: "Package name to uninstall".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("firefox".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "application_control_uninstall",
            "parameters": {
                "package": "firefox"
            }
        })
    }

    fn example_output(&self) -> String {
        "Package firefox uninstalled successfully".to_string()
    }

    fn category(&self) -> &str {
        "application_control"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let package = parameters
            .get("package")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'package' parameter"))?;

        #[cfg(target_os = "windows")]
        {
            let output = std::process::Command::new("winget")
                .args(["uninstall", package, "--silent"])
                .output()?;

            if output.status.success() {
                Ok(format!("Package {} uninstalled successfully", package))
            } else {
                anyhow::bail!(
                    "Failed to uninstall package: {}",
                    String::from_utf8_lossy(&output.stderr)
                )
            }
        }

        #[cfg(target_os = "linux")]
        {
            let output = std::process::Command::new("sudo")
                .args(["apt-get", "remove", "-y", package])
                .output()?;

            if output.status.success() {
                Ok(format!("Package {} uninstalled successfully", package))
            } else {
                anyhow::bail!(
                    "Failed to uninstall package: {}",
                    String::from_utf8_lossy(&output.stderr)
                )
            }
        }

        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("brew")
                .args(["uninstall", package])
                .output()?;

            if output.status.success() {
                Ok(format!("Package {} uninstalled successfully", package))
            } else {
                anyhow::bail!(
                    "Failed to uninstall package: {}",
                    String::from_utf8_lossy(&output.stderr)
                )
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            let _ = package;
            anyhow::bail!("Uninstall not implemented on this platform")
        }
    }
}
