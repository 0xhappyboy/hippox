//! File hash skill

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::io::Read;

use crate::{
    file_exists,
    types::{Skill, SkillParameter},
    validate_path,
};

/// Skill for calculating hash of a file
///
/// # Description
/// Computes cryptographic hash (MD5, SHA256, or SHA512) of a file's contents. This is useful for
/// file integrity verification, checksum validation, and detecting file modifications.
/// The file is read in its entirety to compute the hash.
///
/// # Parameters
/// * `path` (required) - Path to the file to hash
/// * `algorithm` (optional) - Hash algorithm to use: "md5", "sha256" (default), or "sha512"
///
/// # Example
/// ```
/// Input: path="/path/to/file.txt", algorithm="sha256"
/// Output: "SHA256: a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e"
/// ```
#[derive(Debug)]
pub struct HashFileSkill;

#[async_trait::async_trait]
impl Skill for HashFileSkill {
    fn name(&self) -> &str {
        "hash_file"
    }

    fn description(&self) -> &str {
        "Calculate hash (MD5, SHA256, or SHA512) of a file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to compute file checksums for integrity verification"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/file.txt".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "algorithm".to_string(),
                param_type: "string".to_string(),
                description: "Hash algorithm (md5, sha256, sha512)".to_string(),
                required: false,
                default: Some(Value::String("sha256".to_string())),
                example: Some(Value::String("md5".to_string())),
                enum_values: Some(vec![
                    "md5".to_string(),
                    "sha256".to_string(),
                    "sha512".to_string(),
                ]),
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "hash_file",
            "parameters": {
                "path": "/path/to/file.txt",
                "algorithm": "sha256"
            }
        })
    }

    fn example_output(&self) -> String {
        "SHA256: a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e".to_string()
    }

    fn category(&self) -> crate::SkillCategory {
        crate::SkillCategory::Cryptography
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let algorithm = parameters
            .get("algorithm")
            .and_then(|v| v.as_str())
            .unwrap_or("sha256");

        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("File not found: {}", path);
        }

        let mut file = fs::File::open(&validated_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let result = match algorithm {
            "md5" => {
                let digest = md5::compute(&buffer);
                format!("MD5: {:x}", digest)
            }
            "sha256" => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&buffer);
                let result = hasher.finalize();
                format!("SHA256: {}", hex::encode(result))
            }
            "sha512" => {
                use sha2::{Digest, Sha512};
                let mut hasher = Sha512::new();
                hasher.update(&buffer);
                let result = hasher.finalize();
                format!("SHA512: {}", hex::encode(result))
            }
            _ => anyhow::bail!("Unsupported algorithm: {}", algorithm),
        };
        Ok(result)
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        Ok(())
    }
}