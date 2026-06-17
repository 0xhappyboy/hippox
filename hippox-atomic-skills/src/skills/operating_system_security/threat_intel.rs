//! Threat intelligence query skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCallback, SkillCategory, SkillContext, operating_system_security::common::query_threat_intel, types::{Skill, SkillParameter}
};

#[derive(Debug)]
pub struct ThreatIntelSkill;

#[async_trait::async_trait]
impl Skill for ThreatIntelSkill {
    fn name(&self) -> &str {
        "security_threat_intel"
    }

    fn description(&self) -> &str {
        "Query threat intelligence for IP, domain, or file hash"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to check if an IP address, domain name, or file hash is known to be malicious."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "indicator".to_string(),
            param_type: "string".to_string(),
            description: "IP address, domain name, or file hash (MD5, SHA-1, SHA-256)".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("185.130.5.253".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "security_threat_intel",
            "parameters": {
                "indicator": "185.130.5.253"
            }
        })
    }

    fn example_output(&self) -> String {
        "Indicator: 185.130.5.253\nType: ip\nMalicious: Yes\nConfidence: 95%\nThreat Type: malware\nFirst Seen: 2024-01-01\nSource: Internal Threat Intelligence Database".to_string()
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
        let indicator = parameters
            .get("indicator")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'indicator' parameter"))?;
        let result = query_threat_intel(indicator);
        let mut output = String::new();
        output.push_str(&format!("Indicator: {}\n", result.indicator));
        output.push_str(&format!("Type: {}\n", result.indicator_type));
        output.push_str(&format!(
            "Malicious: {}\n",
            if result.malicious { "Yes" } else { "No" }
        ));
        output.push_str(&format!("Confidence: {:.0}%\n", result.confidence * 100.0));
        output.push_str(&format!(
            "Threat Types: {}\n",
            result.threat_type.join(", ")
        ));
        if let Some(first_seen) = result.first_seen {
            output.push_str(&format!("First Seen: {}\n", first_seen));
        }
        if let Some(last_seen) = result.last_seen {
            output.push_str(&format!("Last Seen: {}\n", last_seen));
        }
        if !result.related_indicators.is_empty() {
            output.push_str(&format!(
                "Related Indicators: {}\n",
                result.related_indicators.join(", ")
            ));
        }
        output.push_str(&format!("Source: {}", result.source));
        Ok(output)
    }
}
