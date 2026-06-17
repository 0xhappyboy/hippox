//! Phishing URL detection skill

use crate::{SkillCallback, SkillContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCategory,
    operating_system_security::common::detect_phishing,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct PhishingDetectSkill;

#[async_trait::async_trait]
impl Skill for PhishingDetectSkill {
    fn name(&self) -> &str {
        "security_phishing_detect"
    }

    fn description(&self) -> &str {
        "Detect if a URL is a phishing link"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to analyze a URL for phishing indicators. Checks for suspicious keywords, domain spoofing, URL shorteners, and more."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "url".to_string(),
            param_type: "string".to_string(),
            description: "URL to check for phishing".to_string(),
            required: true,
            default: None,
            example: Some(Value::String(
                "https://secure-login.example.com".to_string(),
            )),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "security_phishing_detect",
            "parameters": {
                "url": "https://secure-login.example.com"
            }
        })
    }

    fn example_output(&self) -> String {
        "URL: https://secure-login.example.com\nPhishing: Yes\nConfidence: 85%\nDomain Reputation: Suspicious\nReasons:\n- Contains suspicious keyword: Common phishing keyword\n- Potential domain spoofing with suspicious keywords".to_string()
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
        let url = parameters
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?;
        let result = detect_phishing(url);
        let mut output = String::new();
        output.push_str(&format!("URL: {}\n", result.url));
        output.push_str(&format!(
            "Phishing: {}\n",
            if result.is_phishing { "Yes" } else { "No" }
        ));
        output.push_str(&format!("Confidence: {:.0}%\n", result.confidence * 100.0));
        output.push_str(&format!(
            "Domain Reputation: {}\n",
            result.domain_reputation
        ));
        if !result.reasons.is_empty() {
            output.push_str("\nReasons:\n");
            for reason in &result.reasons {
                output.push_str(&format!("- {}\n", reason));
            }
        }
        if result.is_phishing {
            output.push_str(
                "\nThis URL appears to be a phishing attempt. Do not enter any credentials!",
            );
        } else {
            output.push_str("\nThis URL appears legitimate based on current analysis.");
        }
        Ok(output)
    }
}
