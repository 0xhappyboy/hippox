//! Security policy assessment skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCallback, SkillCategory, SkillContext, operating_system_security::common::check_security_policies, types::{Skill, SkillParameter}
};

#[derive(Debug)]
pub struct SecurityPolicyCheckSkill;

#[async_trait::async_trait]
impl Skill for SecurityPolicyCheckSkill {
    fn name(&self) -> &str {
        "security_policy_check"
    }

    fn description(&self) -> &str {
        "Assess system security policy compliance"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check if security policies are properly configured."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "security_policy_check"
        })
    }

    fn example_output(&self) -> String {
        "Security Policy Assessment Results:\n password_min_length: Compliant\n mfa_required: Non-compliant (current: false, expected: true)\n...".to_string()
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
        let policies = check_security_policies();
        if policies.is_empty() {
            return Ok("No security policies found".to_string());
        }
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
                if policy.is_compliant {
                    "Compliant"
                } else {
                    "Non-compliant"
                },
                policy.current_value,
                policy.expected_value,
                policy.severity
            ));
            if !policy.is_compliant {
                result.push_str(&format!("   Recommendation: {}\n", policy.recommendation));
            }
        }
        result.push_str(&format!(
            "\nSummary: {} compliant, {} non-compliant",
            compliant_count, non_compliant_count
        ));
        if non_compliant_count > 0 {
            result.push_str(&format!(
                "\n{} policies need attention",
                non_compliant_count
            ));
        } else {
            result.push_str("\nAll policies are compliant!");
        }
        Ok(result)
    }
}
