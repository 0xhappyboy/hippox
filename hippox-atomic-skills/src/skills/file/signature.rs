//! File signature verification skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use super::common::{file_exists, validate_path, verify_file_signature};
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};

#[derive(Debug)]
pub struct FileSignatureSkill;

#[async_trait::async_trait]
impl Skill for FileSignatureSkill {
    fn name(&self) -> &str {
        "file_signature_verify"
    }

    fn description(&self) -> &str {
        "Verify file signature (hash-based integrity check)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to verify if a file matches an expected signature/hash."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the file to verify".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/tmp/file.txt".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "signature".to_string(),
                param_type: "string".to_string(),
                description: "Expected SHA256 signature/hash".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
                )),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "file_signature_verify",
            "parameters": {
                "path": "/tmp/file.txt",
                "signature": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            }
        })
    }

    fn example_output(&self) -> String {
        "File signature verified: /tmp/file.txt matches expected signature".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::File
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let signature = parameters
            .get("signature")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'signature' parameter"))?;

        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("File not found: {}", path);
        }

        let verified = verify_file_signature(&validated_path.to_string_lossy(), signature)?;

        if verified {
            Ok(format!(
                "File signature verified: {} matches expected signature",
                path
            ))
        } else {
            Ok(format!(
                "File signature mismatch: {} does NOT match expected signature",
                path
            ))
        }
    }
}
