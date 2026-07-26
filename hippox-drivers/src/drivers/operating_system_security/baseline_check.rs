//! Security baseline check driver
//!
//! This driver provides functionality to check system against security baseline standards.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_security::common::run_baseline_check,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for checking security baseline
#[derive(Debug)]
pub struct BaselineCheckDriver;
#[async_trait::async_trait]
impl Driver for BaselineCheckDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_baseline_check"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Check system against security baseline standards"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to verify system compliance with security baseline standards"
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "category".to_string(),
                param_type: "string".to_string(),
                description: "Filter results by category (optional)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("Password Policy".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "show_compliant".to_string(),
                param_type: "boolean".to_string(),
                description: "Show compliant checks as well (default: false)".to_string(),
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
            "action": "security_baseline_check",
            "parameters": {
                "show_compliant": true
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Security Baseline Check Results:\n\nPassword Policy: Minimum password length [FAIL]\n  Current: 8, Expected: 12\n  Recommendation: Configure system to meet Minimum password length requirement\n\nSummary: 2 compliant, 6 non-compliant".to_string();
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
        debug!("Executing security_baseline_check driver");
        let category_filter = parameters.get("category").and_then(|v| v.as_str());
        let show_compliant = parameters.get("show_compliant").and_then(|v| v.as_bool()).unwrap_or(false);
        info!("Baseline check: category={:?}, show_compliant={}", category_filter, show_compliant);
        let results = run_baseline_check();
        let filtered: Vec<_> = results
            .iter()
            .filter(|r| if let Some(cat) = category_filter { r.category == cat } else { true })
            .filter(|r| show_compliant || !r.compliant)
            .collect();
        info!("Filtered {} baseline check results", filtered.len());
        let mut output = String::new();
        output.push_str("Security Baseline Check Results:\n\n");
        if filtered.is_empty() {
            output.push_str("No checks match the specified criteria.");
            info!("No baseline checks match the specified criteria");
        } else {
            let mut current_category = String::new();
            for result in filtered {
                if result.category != current_category {
                    current_category = result.category.clone();
                    output.push_str(&format!("{}:\n", current_category));
                }
                let status = if result.compliant { "PASS" } else { "FAIL" };
                output.push_str(&format!("  {}: {} [{}]\n", result.check_name, status, result.severity));
                output.push_str(&format!("    Current: {}, Expected: {}\n", result.current_value, result.expected_value));
                output.push_str(&format!("    Recommendation: {}\n", result.recommendation));
            }
        }
        let compliant_count = results.iter().filter(|r| r.compliant).count();
        let non_compliant_count = results.iter().filter(|r| !r.compliant).count();
        output.push_str(&format!("\nSummary: {} compliant, {} non-compliant", compliant_count, non_compliant_count));
        if non_compliant_count > 0 {
            output.push_str(&format!("\n{} checks require attention", non_compliant_count));
        }
        info!("Baseline check complete: {} compliant, {} non-compliant", compliant_count, non_compliant_count);
        return Ok(output);
    }
}
