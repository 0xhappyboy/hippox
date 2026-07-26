//! Patch detection driver
//!
//! This driver provides functionality to detect missing security patches and updates.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_security::common::check_patch_status,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for detecting missing security patches
#[derive(Debug)]
pub struct PatchDetectDriver;
#[async_trait::async_trait]
impl Driver for PatchDetectDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_patch_detect"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Detect missing security patches and updates"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to check for missing security patches and system updates"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "show_all".to_string(),
                param_type: "boolean".to_string(),
                description: "Show all patches including installed ones (default: false)".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "severity".to_string(),
                param_type: "string".to_string(),
                description: "Filter by severity: critical, high, medium, low".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("critical".to_string())),
                enum_values: Some(vec!["critical".to_string(), "high".to_string(), "medium".to_string(), "low".to_string()]),
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "security_patch_detect",
            "parameters": {
                "show_all": true,
                "severity": "critical"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Patch Detection Results:\n\nTotal checked: 15\nInstalled: 10\nMissing: 5\n\nMissing Patches:\n  - openssl-security (3.0.0) [HIGH]\n  - kernel-update (5.15.0) [CRITICAL]\n\nSummary: 5 patches missing, 3 critical".to_string();
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
        debug!("Executing security_patch_detect driver");
        let show_all = parameters.get("show_all").and_then(|v| v.as_bool()).unwrap_or(false);
        let severity_filter = parameters.get("severity").and_then(|v| v.as_str());
        info!("Patch detection: show_all={}, severity_filter={:?}", show_all, severity_filter);
        let result = check_patch_status();
        let mut output = String::new();
        output.push_str(&format!("Patch Detection Results:\n\nTotal checked: {}\n", result.total_checked));
        output.push_str(&format!("Installed: {}\n", result.installed));
        output.push_str(&format!("Missing: {}\n", result.missing));
        if result.missing == 0 {
            output.push_str("\nAll patches are installed. System is up to date.");
            info!("All patches are installed");
            return Ok(output);
        }
        let missing_patches: Vec<_> = result
            .patches
            .iter()
            .filter(|p| !p.installed)
            .filter(|p| if let Some(sev) = severity_filter { p.severity == sev } else { true })
            .collect();
        if missing_patches.is_empty() {
            output.push_str(&format!("\nNo missing patches match the specified severity filter: {}", severity_filter.unwrap_or("none")));
            info!("No missing patches match the severity filter");
            return Ok(output);
        }
        info!("Found {} missing patches", missing_patches.len());
        output.push_str("\nMissing Patches:\n");
        for patch in &missing_patches {
            let severity_icon = match patch.severity.as_str() {
                "critical" => "[CRITICAL]",
                "high" => "[HIGH]",
                "medium" => "[MEDIUM]",
                _ => "[LOW]",
            };
            output.push_str(&format!("  - {} {} ({})\n", patch.name, severity_icon, patch.version));
            if !patch.description.is_empty() {
                output.push_str(&format!("    {}\n", patch.description));
            }
        }
        let critical_count = missing_patches.iter().filter(|p| p.severity == "critical").count();
        if critical_count > 0 {
            output.push_str(&format!("\nWarning: {} critical patches missing!", critical_count));
        }
        if show_all {
            let installed_patches: Vec<_> = result.patches.iter().filter(|p| p.installed).collect();
            if !installed_patches.is_empty() {
                output.push_str("\n\nInstalled patches:\n");
                for patch in installed_patches.iter().take(10) {
                    output.push_str(&format!("  - {} ({})\n", patch.name, patch.version));
                }
                if installed_patches.len() > 10 {
                    output.push_str(&format!("  ... and {} more\n", installed_patches.len() - 10));
                }
            }
        }
        return Ok(output);
    }
}
