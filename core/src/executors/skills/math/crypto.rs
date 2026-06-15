//! Cryptographic skills module
//!
//! This module provides various cryptographic operations including hashing (MD5, SHA256, SHA512),
//! file hashing, and Base64 encoding/decoding. These skills can be used by the executor system
//! to perform common cryptographic tasks.

use anyhow::Result;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::io::Read;

use crate::executors::{
    file_exists,
    types::{Skill, SkillParameter},
    validate_path,
};

/// Skill for calculating MD5 hash of a string
///
/// # Description
/// Computes the MD5 hash (128-bit) of a given input string and returns it as a hexadecimal string.
/// MD5 is commonly used for checksums and integrity verification, but is not cryptographically secure.
///
/// # Parameters
/// * `input` (required) - The string to hash
///
/// # Example
/// ```
/// Input: "Hello World"
/// Output: "MD5: b10a8db164e0754105b7a99be72e3fe5"
/// ```
#[derive(Debug)]
pub struct HashMd5Skill;

#[async_trait::async_trait]
impl Skill for HashMd5Skill {
    fn name(&self) -> &str {
        "hash_md5"
    }

    fn description(&self) -> &str {
        "Calculate MD5 hash of a string"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to compute MD5 hash for a text string"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "input".to_string(),
            param_type: "string".to_string(),
            description: "Input string to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Hello World".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "hash_md5",
            "parameters": {
                "input": "Hello World"
            }
        })
    }

    fn example_output(&self) -> String {
        "MD5: b10a8db164e0754105b7a99be72e3fe5".to_string()
    }

    fn category(&self) -> &str {
        "crypto"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let input = parameters
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'input' parameter"))?;

        let digest = md5::compute(input.as_bytes());
        Ok(format!("MD5: {:x}", digest))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("input")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: input"))?;
        Ok(())
    }
}

/// Skill for calculating SHA256 hash of a string
///
/// # Description
/// Computes the SHA256 hash (256-bit) of a given input string and returns it as a hexadecimal string.
/// SHA256 is a cryptographically secure hash function commonly used in security applications,
/// digital signatures, and blockchain technologies.
///
/// # Parameters
/// * `input` (required) - The string to hash
///
/// # Example
/// ```
/// Input: "Hello World"
/// Output: "SHA256: a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e"
/// ```
#[derive(Debug)]
pub struct HashSha256Skill;

#[async_trait::async_trait]
impl Skill for HashSha256Skill {
    fn name(&self) -> &str {
        "hash_sha256"
    }

    fn description(&self) -> &str {
        "Calculate SHA256 hash of a string"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to compute SHA256 hash for a text string"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "input".to_string(),
            param_type: "string".to_string(),
            description: "Input string to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Hello World".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "hash_sha256",
            "parameters": {
                "input": "Hello World"
            }
        })
    }

    fn example_output(&self) -> String {
        "SHA256: a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e".to_string()
    }

    fn category(&self) -> &str {
        "crypto"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let input = parameters
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'input' parameter"))?;
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        Ok(format!("SHA256: {}", hex::encode(result)))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("input")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: input"))?;
        Ok(())
    }
}

/// Skill for calculating SHA512 hash of a string
///
/// # Description
/// Computes the SHA512 hash (512-bit) of a given input string and returns it as a hexadecimal string.
/// SHA512 provides stronger security than SHA256 with a larger output size, suitable for
/// applications requiring higher security margins.
///
/// # Parameters
/// * `input` (required) - The string to hash
///
/// # Example
/// ```
/// Input: "Hello World"
/// Output: "SHA512: 2c74fd17edafd80e8447b0d46741ee243b7eb74dd2149a0ab1b9246fb30382f27e853d8585719e0e67cbda0daa8f51671064615d645ae27acb15bfb1447f459b"
/// ```
#[derive(Debug)]
pub struct HashSha512Skill;

#[async_trait::async_trait]
impl Skill for HashSha512Skill {
    fn name(&self) -> &str {
        "hash_sha512"
    }

    fn description(&self) -> &str {
        "Calculate SHA512 hash of a string"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when you need to compute SHA512 hash for a text string"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "input".to_string(),
            param_type: "string".to_string(),
            description: "Input string to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Hello World".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "hash_sha512",
            "parameters": {
                "input": "Hello World"
            }
        })
    }

    fn example_output(&self) -> String {
        "SHA512: 2c74fd17edafd80e8447b0d46741ee243b7eb74dd2149a0ab1b9246fb30382f27e853d8585719e0e67cbda0daa8f51671064615d645ae27acb15bfb1447f459b".to_string()
    }

    fn category(&self) -> &str {
        "crypto"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let input = parameters
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'input' parameter"))?;
        use sha2::{Digest, Sha512};
        let mut hasher = Sha512::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        Ok(format!("SHA512: {}", hex::encode(result)))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("input")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: input"))?;
        Ok(())
    }
}

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

    fn category(&self) -> &str {
        "crypto"
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

/// Skill for Base64 encoding
///
/// # Description
/// Encodes a string to Base64 format. Base64 encoding is commonly used to represent binary data
/// in an ASCII string format, useful for data transmission over text-based protocols like
/// HTTP or email attachments.
///
/// # Parameters
/// * `input` (required) - The string to encode
///
/// # Example
/// ```
/// Input: "Hello World"
/// Output: "Base64: SGVsbG8gV29ybGQ="
/// ```
#[derive(Debug)]
pub struct Base64EncodeSkill;

#[async_trait::async_trait]
impl Skill for Base64EncodeSkill {
    fn name(&self) -> &str {
        "base64_encode"
    }

    fn description(&self) -> &str {
        "Encode a string to Base64"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to convert text to Base64 encoding"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "input".to_string(),
            param_type: "string".to_string(),
            description: "Input string to encode".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("Hello World".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "base64_encode",
            "parameters": {
                "input": "Hello World"
            }
        })
    }

    fn example_output(&self) -> String {
        "Base64: SGVsbG8gV29ybGQ=".to_string()
    }

    fn category(&self) -> &str {
        "crypto"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let input = parameters
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'input' parameter"))?;
        let encoded = STANDARD.encode(input.as_bytes());
        Ok(format!("Base64: {}", encoded))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("input")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: input"))?;
        Ok(())
    }
}

/// Skill for Base64 decoding
///
/// # Description
/// Decodes a Base64 string back to its original text representation. The decoded data must be
/// valid UTF-8; otherwise, an error is returned indicating that the decoded data is not
/// valid UTF-8.
///
/// # Parameters
/// * `input` (required) - The Base64 string to decode
///
/// # Example
/// ```
/// Input: "SGVsbG8gV29ybGQ="
/// Output: "Decoded: Hello World"
/// ```
#[derive(Debug)]
pub struct Base64DecodeSkill;

#[async_trait::async_trait]
impl Skill for Base64DecodeSkill {
    fn name(&self) -> &str {
        "base64_decode"
    }

    fn description(&self) -> &str {
        "Decode a Base64 string to original text"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to decode Base64 encoded text"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "input".to_string(),
            param_type: "string".to_string(),
            description: "Base64 string to decode".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("SGVsbG8gV29ybGQ=".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "base64_decode",
            "parameters": {
                "input": "SGVsbG8gV29ybGQ="
            }
        })
    }

    fn example_output(&self) -> String {
        "Decoded: Hello World".to_string()
    }

    fn category(&self) -> &str {
        "crypto"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let input = parameters
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'input' parameter"))?;
        let decoded = STANDARD
            .decode(input)
            .map_err(|e| anyhow::anyhow!("Invalid Base64 string: {}", e))?;
        let decoded_str = String::from_utf8(decoded)
            .map_err(|e| anyhow::anyhow!("Decoded data is not valid UTF-8: {}", e))?;
        Ok(format!("Decoded: {}", decoded_str))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("input")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: input"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Test MD5 hash calculation for various inputs
    #[tokio::test]
    async fn test_hash_md5_skill() {
        let skill = HashMd5Skill;
        let mut params = HashMap::new();
        // Test normal string
        params.insert(
            "input".to_string(),
            Value::String("Hello World".to_string()),
        );
        let result = skill.execute(&params).await.unwrap();
        assert_eq!(result, "MD5: b10a8db164e0754105b7a99be72e3fe5");
        // Test empty string
        params.insert("input".to_string(), Value::String("".to_string()));
        let result = skill.execute(&params).await.unwrap();
        assert_eq!(result, "MD5: d41d8cd98f00b204e9800998ecf8427e");
        // Test numeric string
        params.insert("input".to_string(), Value::String("12345".to_string()));
        let result = skill.execute(&params).await.unwrap();
        assert_eq!(result, "MD5: 827ccb0eea8a706c4c34a16891f84e7b");
        // Test missing parameter
        let empty_params = HashMap::new();
        let result = skill.execute(&empty_params).await;
        assert!(result.is_err());
    }

    /// Test SHA256 hash calculation for various inputs
    #[tokio::test]
    async fn test_hash_sha256_skill() {
        let skill = HashSha256Skill;
        let mut params = HashMap::new();
        // Test normal string
        params.insert(
            "input".to_string(),
            Value::String("Hello World".to_string()),
        );
        let result = skill.execute(&params).await.unwrap();
        assert_eq!(
            result,
            "SHA256: a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e"
        );
        // Test empty string
        params.insert("input".to_string(), Value::String("".to_string()));
        let result = skill.execute(&params).await.unwrap();
        assert_eq!(
            result,
            "SHA256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        // Test missing parameter
        let empty_params = HashMap::new();
        let result = skill.execute(&empty_params).await;
        assert!(result.is_err());
    }

    /// Test Base64 encoding and decoding roundtrip
    #[tokio::test]
    async fn test_base64_encode_decode_roundtrip() {
        let encode_skill = Base64EncodeSkill;
        let decode_skill = Base64DecodeSkill;
        let test_cases = vec![
            "Hello World",
            "Rust programming language",
            "Base64编码测试",
            "1234567890!@#$%^&*()",
            "",
            "a", // Single character
        ];
        for test_input in test_cases {
            let mut encode_params = HashMap::new();
            encode_params.insert("input".to_string(), Value::String(test_input.to_string()));
            // Encode
            let encoded_result = encode_skill.execute(&encode_params).await.unwrap();
            // Extract the encoded string (remove "Base64: " prefix)
            let encoded = encoded_result.trim_start_matches("Base64: ");
            // Decode
            let mut decode_params = HashMap::new();
            decode_params.insert("input".to_string(), Value::String(encoded.to_string()));
            let decoded_result = decode_skill.execute(&decode_params).await.unwrap();
            let decoded = decoded_result.trim_start_matches("Decoded: ");
            // Verify roundtrip
            assert_eq!(test_input, decoded, "Failed for input: {}", test_input);
        }
    }
}
