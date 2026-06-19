//! Security baseline check Driver

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    DriverCallback, DriverCategory, DriverContext, operating_system_security::common::run_baseline_check, types::{Driver, DriverParameter}
};

#[derive(Debug)]
pub struct BaselineCheckDriver;

#[async_trait::async_trait]
impl Driver for BaselineCheckDriver {
    fn name(&self) -> &str {
        "security_baseline_check"
    }

    fn description(&self) -> &str {
        "Check system against security baseline standards"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to verify system compliance with security baseline standards"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
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
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "security_baseline_check",
            "parameters": {
                "show_compliant": true
            }
        })
    }

    fn example_output(&self) -> String {
        "Security Baseline Check Results:\n\nPassword Policy: Minimum password length [FAIL]\n  Current: 8, Expected: 12\n  Recommendation: Configure system to meet Minimum password length requirement\n\nSummary: 2 compliant, 6 non-compliant".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::OperatingSystemSecurity
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let category_filter = parameters.get("category").and_then(|v| v.as_str());
        let show_compliant = parameters
            .get("show_compliant")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let results = run_baseline_check();

        let filtered: Vec<_> = results
            .iter()
            .filter(|r| {
                if let Some(cat) = category_filter {
                    r.category == cat
                } else {
                    true
                }
            })
            .filter(|r| show_compliant || !r.compliant)
            .collect();

        let mut output = String::new();
        output.push_str("Security Baseline Check Results:\n\n");

        if filtered.is_empty() {
            output.push_str("No checks match the specified criteria.");
        } else {
            let mut current_category = String::new();
            for result in filtered {
                if result.category != current_category {
                    current_category = result.category.clone();
                    output.push_str(&format!("{}:\n", current_category));
                }

                let status = if result.compliant { "PASS" } else { "FAIL" };
                output.push_str(&format!(
                    "  {}: {} [{}]\n",
                    result.check_name, status, result.severity
                ));
                output.push_str(&format!(
                    "    Current: {}, Expected: {}\n",
                    result.current_value, result.expected_value
                ));
                output.push_str(&format!("    Recommendation: {}\n", result.recommendation));
            }
        }
        let compliant_count = results.iter().filter(|r| r.compliant).count();
        let non_compliant_count = results.iter().filter(|r| !r.compliant).count();
        output.push_str(&format!(
            "\nSummary: {} compliant, {} non-compliant",
            compliant_count, non_compliant_count
        ));
        if non_compliant_count > 0 {
            output.push_str(&format!(
                "\n{} checks require attention",
                non_compliant_count
            ));
        }
        Ok(output)
    }
}
