//! File hash calculation skills

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    calculate_all_hashes, calculate_md5, calculate_sha1, calculate_sha256, calculate_sha512,
    file_exists, validate_path,
};
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};

/// Skill for calculating MD5 hash of a file
#[derive(Debug)]
pub struct HashMd5Skill;

#[async_trait::async_trait]
impl Skill for HashMd5Skill {
    fn name(&self) -> &str {
        "hash_md5"
    }

    fn description(&self) -> &str {
        "Calculate MD5 hash of a file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the MD5 checksum of a file for integrity verification."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the file to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/tmp/file.txt".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "hash_md5",
            "parameters": {
                "path": "/tmp/file.txt"
            }
        })
    }

    fn example_output(&self) -> String {
        "MD5 hash of /tmp/file.txt: d41d8cd98f00b204e9800998ecf8427e".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::File
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("File not found: {}", path);
        }
        let hash = calculate_md5(&validated_path.to_string_lossy())?;
        Ok(format!("MD5 hash of {}: {}", path, hash))
    }
}

/// Skill for calculating SHA1 hash of a file
#[derive(Debug)]
pub struct HashSha1Skill;

#[async_trait::async_trait]
impl Skill for HashSha1Skill {
    fn name(&self) -> &str {
        "hash_sha1"
    }

    fn description(&self) -> &str {
        "Calculate SHA1 hash of a file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the SHA1 checksum of a file for integrity verification."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the file to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/tmp/file.txt".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "hash_sha1",
            "parameters": {
                "path": "/tmp/file.txt"
            }
        })
    }

    fn example_output(&self) -> String {
        "SHA1 hash of /tmp/file.txt: da39a3ee5e6b4b0d3255bfef95601890afd80709".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::File
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("File not found: {}", path);
        }
        let hash = calculate_sha1(&validated_path.to_string_lossy())?;
        Ok(format!("SHA1 hash of {}: {}", path, hash))
    }
}

/// Skill for calculating SHA256 hash of a file
#[derive(Debug)]
pub struct HashSha256Skill;

#[async_trait::async_trait]
impl Skill for HashSha256Skill {
    fn name(&self) -> &str {
        "hash_sha256"
    }

    fn description(&self) -> &str {
        "Calculate SHA256 hash of a file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the SHA256 checksum of a file for integrity verification."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the file to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/tmp/file.txt".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "hash_sha256",
            "parameters": {
                "path": "/tmp/file.txt"
            }
        })
    }

    fn example_output(&self) -> String {
        "SHA256 hash of /tmp/file.txt: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::File
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("File not found: {}", path);
        }
        let hash = calculate_sha256(&validated_path.to_string_lossy())?;
        Ok(format!("SHA256 hash of {}: {}", path, hash))
    }
}

/// Skill for calculating SHA512 hash of a file
#[derive(Debug)]
pub struct HashSha512Skill;

#[async_trait::async_trait]
impl Skill for HashSha512Skill {
    fn name(&self) -> &str {
        "hash_sha512"
    }

    fn description(&self) -> &str {
        "Calculate SHA512 hash of a file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get the SHA512 checksum of a file for integrity verification."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the file to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/tmp/file.txt".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "hash_sha512",
            "parameters": {
                "path": "/tmp/file.txt"
            }
        })
    }

    fn example_output(&self) -> String {
        "SHA512 hash of /tmp/file.txt: cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::File
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("File not found: {}", path);
        }
        let hash = calculate_sha512(&validated_path.to_string_lossy())?;
        Ok(format!("SHA512 hash of {}: {}", path, hash))
    }
}

/// Skill for calculating all hashes of a file
#[derive(Debug)]
pub struct HashFileSkill;

#[async_trait::async_trait]
impl Skill for HashFileSkill {
    fn name(&self) -> &str {
        "hash_file"
    }

    fn description(&self) -> &str {
        "Calculate all hashes (MD5, SHA1, SHA256, SHA512, BLAKE3) of a file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to get comprehensive hash information for a file."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the file to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/tmp/file.txt".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "hash_file",
            "parameters": {
                "path": "/tmp/file.txt"
            }
        })
    }

    fn example_output(&self) -> String {
        "File: /tmp/file.txt\nMD5: d41d8cd98f00b204e9800998ecf8427e\nSHA1: da39a3ee5e6b4b0d3255bfef95601890afd80709\nSHA256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\nSHA512: cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e\nBLAKE3: ...".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::File
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("File not found: {}", path);
        }
        let result = calculate_all_hashes(&validated_path.to_string_lossy())?;

        let mut output = format!("File: {}\n", result.path);
        if let Some(hash) = result.md5 {
            output.push_str(&format!("MD5: {}\n", hash));
        }
        if let Some(hash) = result.sha1 {
            output.push_str(&format!("SHA1: {}\n", hash));
        }
        if let Some(hash) = result.sha256 {
            output.push_str(&format!("SHA256: {}\n", hash));
        }
        if let Some(hash) = result.sha512 {
            output.push_str(&format!("SHA512: {}\n", hash));
        }
        if let Some(hash) = result.blake3 {
            output.push_str(&format!("BLAKE3: {}\n", hash));
        }

        Ok(output)
    }
}
