//! Redis database driver module
//!
//! This module provides drivers for Redis operations including key-value
//! storage, retrieval, deletion, and hash operations.
use redis::{Client, Commands, Connection};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::DriverCategory;
use crate::types::{Driver, DriverParameter};
use crate::{DriverCallback, DriverContext, DriverError, DriverResult};
/// Creates a Redis connection
///
/// # Arguments
/// * `host` - Redis server hostname
/// * `port` - Redis server port
/// * `password` - Redis password (empty if none)
/// * `db` - Redis database number
///
/// # Returns
/// * `DriverResult<Connection>` - Redis connection on success
fn get_redis_connection(host: &str, port: u16, password: &str, db: usize) -> DriverResult<Connection> {
    let url = if password.is_empty() { format!("redis://{}:{}/", host, port) } else { format!("redis://:{}@{}:{}/{}", password, host, port, db) };
    debug!("Connecting to Redis at {}:{}", host, port);
    let client = Client::open(url).map_err(|e| DriverError::execution(format!("Failed to create Redis client: {}", e)))?;
    let conn = client.get_connection().map_err(|e| DriverError::execution(format!("Failed to connect to Redis: {}", e)))?;
    info!("Successfully connected to Redis at {}:{}", host, port);
    return Ok(conn);
}
/// Retrieves a string parameter from the parameters map
///
/// # Arguments
/// * `params` - Parameters map
/// * `name` - Parameter name
///
/// # Returns
/// * `DriverResult<String>` - Parameter value on success
fn get_param_string(params: &HashMap<String, Value>, name: &str) -> DriverResult<String> {
    return params.get(name).and_then(|v| v.as_str()).map(|s| s.to_string()).ok_or_else(|| DriverError::missing_parameter(name));
}
/// Retrieves a u64 parameter from the parameters map with a default value
///
/// # Arguments
/// * `params` - Parameters map
/// * `name` - Parameter name
/// * `default` - Default value if parameter is not present
///
/// # Returns
/// * `u64` - Parameter value or default
fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    return params.get(name).and_then(|v| v.as_u64()).unwrap_or(default);
}
/// Retrieves a usize parameter from the parameters map with a default value
///
/// # Arguments
/// * `params` - Parameters map
/// * `name` - Parameter name
/// * `default` - Default value if parameter is not present
///
/// # Returns
/// * `usize` - Parameter value or default
fn get_param_usize(params: &HashMap<String, Value>, name: &str, default: usize) -> usize {
    return params.get(name).and_then(|v| v.as_u64()).map(|v| v as usize).unwrap_or(default);
}
/// Driver for setting a key-value pair in Redis
#[derive(Debug)]
pub struct RedisSetDriver;
#[async_trait::async_trait]
impl Driver for RedisSetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "redis_set";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Set a key-value pair in Redis";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to store data in Redis";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Database;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Redis host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Redis port".to_string(),
                required: false,
                default: Some(Value::Number(6379.into())),
                example: Some(Value::Number(6379.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Redis password".to_string(),
                required: false,
                default: Some(Value::String("".to_string())),
                example: Some(Value::String("".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "db".to_string(),
                param_type: "integer".to_string(),
                description: "Redis database number".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(0.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Redis key".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user:100".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "value".to_string(),
                param_type: "string".to_string(),
                description: "Value to store".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("John Doe".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "ttl".to_string(),
                param_type: "integer".to_string(),
                description: "Time to live in seconds".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(3600.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "redis_set",
            "parameters": {
                "host": "localhost",
                "key": "user:100",
                "value": "John Doe",
                "ttl": 3600
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Successfully set key 'user:100'".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing redis_set driver");
        // Extract required parameters
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 6379) as u16;
        let password = parameters.get("password").and_then(|v| v.as_str()).unwrap_or("");
        let db = get_param_usize(parameters, "db", 0);
        let key = get_param_string(parameters, "key")?;
        let value = get_param_string(parameters, "value")?;
        let ttl = parameters.get("ttl").and_then(|v| v.as_u64());
        let mut conn = get_redis_connection(&host, port, password, db)?;
        debug!("Setting key: {}", key);
        let _: () = conn.set(&key, &value).map_err(|e| DriverError::execution(format!("Failed to set key: {}", e)))?;
        if let Some(ttl_secs) = ttl {
            debug!("Setting TTL for key {}: {} seconds", key, ttl_secs);
            let _: () = conn.expire(&key, ttl_secs as i64).map_err(|e| DriverError::execution(format!("Failed to set TTL: {}", e)))?;
        }
        info!("Successfully set key '{}'", key);
        return Ok(format!("Successfully set key '{}'", key));
    }
}
/// Driver for getting a value from Redis by key
#[derive(Debug)]
pub struct RedisGetDriver;
#[async_trait::async_trait]
impl Driver for RedisGetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "redis_get";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get a value from Redis by key";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to retrieve data from Redis";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Database;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Redis host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Redis port".to_string(),
                required: false,
                default: Some(Value::Number(6379.into())),
                example: Some(Value::Number(6379.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Redis password".to_string(),
                required: false,
                default: Some(Value::String("".to_string())),
                example: Some(Value::String("".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "db".to_string(),
                param_type: "integer".to_string(),
                description: "Redis database number".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(0.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Redis key".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user:100".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "redis_get",
            "parameters": {
                "host": "localhost",
                "key": "user:100"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "John Doe".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing redis_get driver");
        // Extract required parameters
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 6379) as u16;
        let password = parameters.get("password").and_then(|v| v.as_str()).unwrap_or("");
        let db = get_param_usize(parameters, "db", 0);
        let key = get_param_string(parameters, "key")?;
        let mut conn = get_redis_connection(&host, port, password, db)?;
        debug!("Getting key: {}", key);
        let value: Option<String> = conn.get(&key).map_err(|e| DriverError::execution(format!("Failed to get key: {}", e)))?;
        match value {
            Some(v) => {
                info!("Successfully retrieved key '{}'", key);
                return Ok(v);
            }
            None => {
                info!("Key '{}' not found", key);
                return Ok("null".to_string());
            }
        }
    }
}
/// Driver for deleting a key from Redis
#[derive(Debug)]
pub struct RedisDelDriver;
#[async_trait::async_trait]
impl Driver for RedisDelDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "redis_del";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Delete a key from Redis";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to delete data from Redis";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Database;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Redis host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Redis port".to_string(),
                required: false,
                default: Some(Value::Number(6379.into())),
                example: Some(Value::Number(6379.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Redis password".to_string(),
                required: false,
                default: Some(Value::String("".to_string())),
                example: Some(Value::String("".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "db".to_string(),
                param_type: "integer".to_string(),
                description: "Redis database number".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(0.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Redis key to delete".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user:100".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "redis_del",
            "parameters": {
                "host": "localhost",
                "key": "user:100"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Successfully deleted key 'user:100'".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing redis_del driver");
        // Extract required parameters
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 6379) as u16;
        let password = parameters.get("password").and_then(|v| v.as_str()).unwrap_or("");
        let db = get_param_usize(parameters, "db", 0);
        let key = get_param_string(parameters, "key")?;
        let mut conn = get_redis_connection(&host, port, password, db)?;
        debug!("Deleting key: {}", key);
        let deleted: i32 = conn.del(&key).map_err(|e| DriverError::execution(format!("Failed to delete key: {}", e)))?;
        if deleted > 0 {
            info!("Successfully deleted key '{}'", key);
            return Ok(format!("Successfully deleted key '{}'", key));
        } else {
            info!("Key '{}' not found", key);
            return Ok(format!("Key '{}' not found", key));
        }
    }
}
/// Driver for finding keys matching a pattern in Redis
#[derive(Debug)]
pub struct RedisKeysDriver;
#[async_trait::async_trait]
impl Driver for RedisKeysDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "redis_keys";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Find keys matching a pattern in Redis";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to list keys in Redis";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Database;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Redis host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Redis port".to_string(),
                required: false,
                default: Some(Value::Number(6379.into())),
                example: Some(Value::Number(6379.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Redis password".to_string(),
                required: false,
                default: Some(Value::String("".to_string())),
                example: Some(Value::String("".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "db".to_string(),
                param_type: "integer".to_string(),
                description: "Redis database number".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(0.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Key pattern (e.g., 'user:*')".to_string(),
                required: false,
                default: Some(Value::String("*".to_string())),
                example: Some(Value::String("user:*".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "redis_keys",
            "parameters": {
                "host": "localhost",
                "pattern": "user:*"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"["user:100", "user:101"]"#.to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing redis_keys driver");
        // Extract required parameters
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 6379) as u16;
        let password = parameters.get("password").and_then(|v| v.as_str()).unwrap_or("");
        let db = get_param_usize(parameters, "db", 0);
        let pattern = parameters.get("pattern").and_then(|v| v.as_str()).unwrap_or("*");
        let mut conn = get_redis_connection(&host, port, password, db)?;
        debug!("Searching keys with pattern: {}", pattern);
        let keys: Vec<String> = conn.keys(pattern).map_err(|e| DriverError::execution(format!("Failed to list keys: {}", e)))?;
        info!("Found {} keys matching pattern", keys.len());
        return Ok(json!(keys).to_string());
    }
}
/// Driver for setting a field in a Redis hash
#[derive(Debug)]
pub struct RedisHSetDriver;
#[async_trait::async_trait]
impl Driver for RedisHSetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "redis_hset";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Set a field in a Redis hash";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to store structured data in Redis";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Database;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Redis host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Redis port".to_string(),
                required: false,
                default: Some(Value::Number(6379.into())),
                example: Some(Value::Number(6379.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Redis password".to_string(),
                required: false,
                default: Some(Value::String("".to_string())),
                example: Some(Value::String("".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "db".to_string(),
                param_type: "integer".to_string(),
                description: "Redis database number".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(0.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Hash key".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user:100".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "field".to_string(),
                param_type: "string".to_string(),
                description: "Field name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("name".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "value".to_string(),
                param_type: "string".to_string(),
                description: "Value to set".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("John Doe".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "redis_hset",
            "parameters": {
                "host": "localhost",
                "key": "user:100",
                "field": "name",
                "value": "John Doe"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Successfully set field 'name' in hash 'user:100'".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing redis_hset driver");
        // Extract required parameters
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 6379) as u16;
        let password = parameters.get("password").and_then(|v| v.as_str()).unwrap_or("");
        let db = get_param_usize(parameters, "db", 0);
        let key = get_param_string(parameters, "key")?;
        let field = get_param_string(parameters, "field")?;
        let value = get_param_string(parameters, "value")?;
        let mut conn = get_redis_connection(&host, port, password, db)?;
        debug!("Setting hash field: {}:{} = {}", key, field, value);
        let _: i32 = conn.hset(&key, &field, &value).map_err(|e| DriverError::execution(format!("Failed to set hash field: {}", e)))?;
        info!("Successfully set field '{}' in hash '{}'", field, key);
        return Ok(format!("Successfully set field '{}' in hash '{}'", field, key));
    }
}
/// Driver for getting a field from a Redis hash
#[derive(Debug)]
pub struct RedisHGetDriver;
#[async_trait::async_trait]
impl Driver for RedisHGetDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "redis_hget";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get a field from a Redis hash";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to retrieve structured data from Redis";
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Database;
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Redis host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Redis port".to_string(),
                required: false,
                default: Some(Value::Number(6379.into())),
                example: Some(Value::Number(6379.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Redis password".to_string(),
                required: false,
                default: Some(Value::String("".to_string())),
                example: Some(Value::String("".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "db".to_string(),
                param_type: "integer".to_string(),
                description: "Redis database number".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(0.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Hash key".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user:100".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "field".to_string(),
                param_type: "string".to_string(),
                description: "Field name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("name".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "redis_hget",
            "parameters": {
                "host": "localhost",
                "key": "user:100",
                "field": "name"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "John Doe".to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing redis_hget driver");
        // Extract required parameters
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 6379) as u16;
        let password = parameters.get("password").and_then(|v| v.as_str()).unwrap_or("");
        let db = get_param_usize(parameters, "db", 0);
        let key = get_param_string(parameters, "key")?;
        let field = get_param_string(parameters, "field")?;
        let mut conn = get_redis_connection(&host, port, password, db)?;
        debug!("Getting hash field: {}:{}", key, field);
        let value: Option<String> = conn.hget(&key, &field).map_err(|e| DriverError::execution(format!("Failed to get hash field: {}", e)))?;
        match value {
            Some(v) => {
                info!("Successfully retrieved field '{}' from hash '{}'", field, key);
                return Ok(v);
            }
            None => {
                info!("Field '{}' not found in hash '{}'", field, key);
                return Ok("null".to_string());
            }
        }
    }
}
