//! File hash calculation skills
//!
//! This module provides drivers for calculating various cryptographic
//! hashes of files including MD5, SHA1, SHA256, SHA512, and BLAKE3.
use crate::DriverCallback;
use crate::DriverContext;
use crate::DriverResult;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{calculate_all_hashes, calculate_md5, calculate_sha1, calculate_sha256, calculate_sha512, file_exists, validate_path};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info, warn};
/// Driver for calculating MD5 hash of a file
#[derive(Debug)]
pub struct HashMd5Driver;
#[async_trait::async_trait]
impl Driver for HashMd5Driver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "hash_md5"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Calculate MD5 hash of a file"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the MD5 checksum of a file for integrity verification."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the file to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/tmp/file.txt".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "hash_md5",
            "parameters": {
                "path": "/tmp/file.txt"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "MD5 hash of /tmp/file.txt: d41d8cd98f00b204e9800998ecf8427e".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::File;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing hash_md5 driver");
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'path' parameter");
            return crate::DriverError::missing_parameter("path");
        })?;
        debug!("Calculating MD5 hash for: {}", path);
        let validated_path = validate_path(path, None).map_err(|e| {
            debug!("Failed to validate path: {}", e);
            return crate::DriverError::execution(format!("Failed to validate path: {}", e));
        })?;
        if !file_exists(&validated_path.to_string_lossy()) {
            warn!("File not found: {}", path);
            return Err(crate::DriverError::execution(format!("File not found: {}", path)));
        }
        let hash = calculate_md5(&validated_path.to_string_lossy()).map_err(|e| {
            debug!("Failed to calculate MD5 hash: {}", e);
            return crate::DriverError::execution(format!("Failed to calculate MD5 hash: {}", e));
        })?;
        info!("MD5 hash calculated for {}", path);
        return Ok(format!("MD5 hash of {}: {}", path, hash));
    }
}
/// Driver for calculating SHA1 hash of a file
#[derive(Debug)]
pub struct HashSha1Driver;
#[async_trait::async_trait]
impl Driver for HashSha1Driver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "hash_sha1"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Calculate SHA1 hash of a file"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the SHA1 checksum of a file for integrity verification."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the file to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/tmp/file.txt".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "hash_sha1",
            "parameters": {
                "path": "/tmp/file.txt"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "SHA1 hash of /tmp/file.txt: da39a3ee5e6b4b0d3255bfef95601890afd80709".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::File;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing hash_sha1 driver");
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'path' parameter");
            return crate::DriverError::missing_parameter("path");
        })?;
        debug!("Calculating SHA1 hash for: {}", path);
        let validated_path = validate_path(path, None).map_err(|e| {
            debug!("Failed to validate path: {}", e);
            return crate::DriverError::execution(format!("Failed to validate path: {}", e));
        })?;
        if !file_exists(&validated_path.to_string_lossy()) {
            warn!("File not found: {}", path);
            return Err(crate::DriverError::execution(format!("File not found: {}", path)));
        }
        let hash = calculate_sha1(&validated_path.to_string_lossy()).map_err(|e| {
            debug!("Failed to calculate SHA1 hash: {}", e);
            return crate::DriverError::execution(format!("Failed to calculate SHA1 hash: {}", e));
        })?;
        info!("SHA1 hash calculated for {}", path);
        return Ok(format!("SHA1 hash of {}: {}", path, hash));
    }
}
/// Driver for calculating SHA256 hash of a file
#[derive(Debug)]
pub struct HashSha256Driver;
#[async_trait::async_trait]
impl Driver for HashSha256Driver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "hash_sha256"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Calculate SHA256 hash of a file"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the SHA256 checksum of a file for integrity verification."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the file to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/tmp/file.txt".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "hash_sha256",
            "parameters": {
                "path": "/tmp/file.txt"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "SHA256 hash of /tmp/file.txt: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::File;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing hash_sha256 driver");
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'path' parameter");
            return crate::DriverError::missing_parameter("path");
        })?;
        debug!("Calculating SHA256 hash for: {}", path);
        let validated_path = validate_path(path, None).map_err(|e| {
            debug!("Failed to validate path: {}", e);
            return crate::DriverError::execution(format!("Failed to validate path: {}", e));
        })?;
        if !file_exists(&validated_path.to_string_lossy()) {
            warn!("File not found: {}", path);
            return Err(crate::DriverError::execution(format!("File not found: {}", path)));
        }
        let hash = calculate_sha256(&validated_path.to_string_lossy()).map_err(|e| {
            debug!("Failed to calculate SHA256 hash: {}", e);
            return crate::DriverError::execution(format!("Failed to calculate SHA256 hash: {}", e));
        })?;
        info!("SHA256 hash calculated for {}", path);
        return Ok(format!("SHA256 hash of {}: {}", path, hash));
    }
}
/// Driver for calculating SHA512 hash of a file
#[derive(Debug)]
pub struct HashSha512Driver;
#[async_trait::async_trait]
impl Driver for HashSha512Driver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "hash_sha512"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Calculate SHA512 hash of a file"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get the SHA512 checksum of a file for integrity verification."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the file to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/tmp/file.txt".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "hash_sha512",
            "parameters": {
                "path": "/tmp/file.txt"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "SHA512 hash of /tmp/file.txt: cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::File;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing hash_sha512 driver");
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'path' parameter");
            return crate::DriverError::missing_parameter("path");
        })?;
        debug!("Calculating SHA512 hash for: {}", path);
        let validated_path = validate_path(path, None).map_err(|e| {
            debug!("Failed to validate path: {}", e);
            return crate::DriverError::execution(format!("Failed to validate path: {}", e));
        })?;
        if !file_exists(&validated_path.to_string_lossy()) {
            warn!("File not found: {}", path);
            return Err(crate::DriverError::execution(format!("File not found: {}", path)));
        }
        let hash = calculate_sha512(&validated_path.to_string_lossy()).map_err(|e| {
            debug!("Failed to calculate SHA512 hash: {}", e);
            return crate::DriverError::execution(format!("Failed to calculate SHA512 hash: {}", e));
        })?;
        info!("SHA512 hash calculated for {}", path);
        return Ok(format!("SHA512 hash of {}: {}", path, hash));
    }
}
/// Driver for calculating all hashes of a file
#[derive(Debug)]
pub struct HashFileDriver;
#[async_trait::async_trait]
impl Driver for HashFileDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "hash_file"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Calculate all hashes (MD5, SHA1, SHA256, SHA512, BLAKE3) of a file"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to get comprehensive hash information for a file."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the file to hash".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/tmp/file.txt".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "hash_file",
            "parameters": {
                "path": "/tmp/file.txt"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "File: /tmp/file.txt\nMD5: d41d8cd98f00b204e9800998ecf8427e\nSHA1: da39a3ee5e6b4b0d3255bfef95601890afd80709\nSHA256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\nSHA512: cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e\nBLAKE3: ...".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::File;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing hash_file driver");
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
            debug!("Missing 'path' parameter");
            return crate::DriverError::missing_parameter("path");
        })?;
        debug!("Calculating all hashes for: {}", path);
        let validated_path = validate_path(path, None).map_err(|e| {
            debug!("Failed to validate path: {}", e);
            return crate::DriverError::execution(format!("Failed to validate path: {}", e));
        })?;
        if !file_exists(&validated_path.to_string_lossy()) {
            warn!("File not found: {}", path);
            return Err(crate::DriverError::execution(format!("File not found: {}", path)));
        }
        let result = calculate_all_hashes(&validated_path.to_string_lossy()).map_err(|e| {
            debug!("Failed to calculate hashes: {}", e);
            return crate::DriverError::execution(format!("Failed to calculate hashes: {}", e));
        })?;
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
        info!("All hashes calculated for {}", path);
        return Ok(output);
    }
}
