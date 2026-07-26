//! Security policy assessment driver
//!
//! This driver provides functionality to assess system security policy compliance.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    operating_system_security::common::check_security_policies,
    types::{Driver, DriverParameter},
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for checking security policies
#[derive(Debug)]
pub struct SecurityPolicyCheckDriver;
#[async_trait::async_trait]
impl Driver for SecurityPolicyCheckDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "security_policy_check"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Assess system security policy compliance"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to check if security policies are properly configured."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "security_policy_check"
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Security Policy Assessment Results:\n password_min_length: Compliant\n mfa_required: Non-compliant (current: false, expected: true)\n...".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::OperatingSystemSecurity;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        _parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing security_policy_check driver");
        let policies = check_security_policies();
        if policies.is_empty() {
            info!("No security policies found");
            return Ok("No security policies found".to_string());
        }
        info!("Checking {} security policies", policies.len());
        let mut result = "Security Policy Assessment Results:\n\n".to_string();
        let mut compliant_count = 0;
        let mut non_compliant_count = 0;
        for policy in policies {
            let status = if policy.is_compliant {
                compliant_count += 1;
                "Yes"
            } else {
                non_compliant_count += 1;
                "No"
            };
            result.push_str(&format!(
                "{} {}: {} (current: {}, expected: {}) [Severity: {}]\n",
                status,
                policy.policy_name,
                if policy.is_compliant { "Compliant" } else { "Non-compliant" },
                policy.current_value,
                policy.expected_value,
                policy.severity
            ));
            if !policy.is_compliant {
                result.push_str(&format!("   Recommendation: {}\n", policy.recommendation));
            }
        }
        result.push_str(&format!("\nSummary: {} compliant, {} non-compliant", compliant_count, non_compliant_count));
        if non_compliant_count > 0 {
            result.push_str(&format!("\n{} policies need attention", non_compliant_count));
            info!("Found {} non-compliant policies", non_compliant_count);
        } else {
            result.push_str("\nAll policies are compliant!");
            info!("All policies are compliant");
        }
        return Ok(result);
    }
}
