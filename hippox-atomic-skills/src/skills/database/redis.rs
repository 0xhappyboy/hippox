use anyhow::Result;
use redis::{Client, Commands, Connection};
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};

fn get_redis_connection(host: &str, port: u16, password: &str, db: usize) -> Result<Connection> {
    let url = if password.is_empty() {
        format!("redis://{}:{}/", host, port)
    } else {
        format!("redis://:{}@{}:{}/{}", password, host, port, db)
    };
    let client = Client::open(url)?;
    Ok(client.get_connection()?)
}

fn get_param_string(params: &HashMap<String, Value>, name: &str) -> Result<String> {
    params
        .get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: {}", name))
}

fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    params.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
}

fn get_param_usize(params: &HashMap<String, Value>, name: &str, default: usize) -> usize {
    params
        .get(name)
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(default)
}

/// Redis Set Skill
#[derive(Debug)]
pub struct RedisSetSkill;

#[async_trait::async_trait]
impl Skill for RedisSetSkill {
    fn name(&self) -> &str {
        "redis_set"
    }
    fn description(&self) -> &str {
        "Set a key-value pair in Redis"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to store data in Redis"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::Database
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Redis host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Redis port".to_string(),
                required: false,
                default: Some(Value::Number(6379.into())),
                example: Some(Value::Number(6379.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Redis password".to_string(),
                required: false,
                default: Some(Value::String("".to_string())),
                example: Some(Value::String("".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "db".to_string(),
                param_type: "integer".to_string(),
                description: "Redis database number".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(0.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Redis key".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user:100".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "value".to_string(),
                param_type: "string".to_string(),
                description: "Value to store".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("John Doe".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "ttl".to_string(),
                param_type: "integer".to_string(),
                description: "Time to live in seconds".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(3600.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "redis_set", "parameters": { "host": "localhost", "key": "user:100", "value": "John Doe", "ttl": 3600 } })
    }

    fn example_output(&self) -> String {
        "Successfully set key 'user:100'".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 6379) as u16;
        let password = parameters
            .get("password")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let db = get_param_usize(parameters, "db", 0);
        let key = get_param_string(parameters, "key")?;
        let value = get_param_string(parameters, "value")?;
        let ttl = parameters.get("ttl").and_then(|v| v.as_u64());

        let mut conn = get_redis_connection(&host, port, password, db)?;
        let _: () = conn.set(&key, &value)?;
        if let Some(ttl_secs) = ttl {
            let _: () = conn.expire(&key, ttl_secs as i64)?;
        }
        Ok(format!("Successfully set key '{}'", key))
    }
}

/// Redis Get Skill
#[derive(Debug)]
pub struct RedisGetSkill;

#[async_trait::async_trait]
impl Skill for RedisGetSkill {
    fn name(&self) -> &str {
        "redis_get"
    }
    fn description(&self) -> &str {
        "Get a value from Redis by key"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to retrieve data from Redis"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::Database
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Redis host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Redis port".to_string(),
                required: false,
                default: Some(Value::Number(6379.into())),
                example: Some(Value::Number(6379.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Redis password".to_string(),
                required: false,
                default: Some(Value::String("".to_string())),
                example: Some(Value::String("".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "db".to_string(),
                param_type: "integer".to_string(),
                description: "Redis database number".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(0.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Redis key".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user:100".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "redis_get", "parameters": { "host": "localhost", "key": "user:100" } })
    }

    fn example_output(&self) -> String {
        "John Doe".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 6379) as u16;
        let password = parameters
            .get("password")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let db = get_param_usize(parameters, "db", 0);
        let key = get_param_string(parameters, "key")?;

        let mut conn = get_redis_connection(&host, port, password, db)?;
        let value: Option<String> = conn.get(&key)?;
        match value {
            Some(v) => Ok(v),
            None => Ok("null".to_string()),
        }
    }
}

/// Redis Delete Skill
#[derive(Debug)]
pub struct RedisDelSkill;

#[async_trait::async_trait]
impl Skill for RedisDelSkill {
    fn name(&self) -> &str {
        "redis_del"
    }
    fn description(&self) -> &str {
        "Delete a key from Redis"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to delete data from Redis"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::Database
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Redis host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Redis port".to_string(),
                required: false,
                default: Some(Value::Number(6379.into())),
                example: Some(Value::Number(6379.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Redis password".to_string(),
                required: false,
                default: Some(Value::String("".to_string())),
                example: Some(Value::String("".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "db".to_string(),
                param_type: "integer".to_string(),
                description: "Redis database number".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(0.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Redis key to delete".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user:100".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "redis_del", "parameters": { "host": "localhost", "key": "user:100" } })
    }

    fn example_output(&self) -> String {
        "Successfully deleted key 'user:100'".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 6379) as u16;
        let password = parameters
            .get("password")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let db = get_param_usize(parameters, "db", 0);
        let key = get_param_string(parameters, "key")?;

        let mut conn = get_redis_connection(&host, port, password, db)?;
        let deleted: i32 = conn.del(&key)?;
        if deleted > 0 {
            Ok(format!("Successfully deleted key '{}'", key))
        } else {
            Ok(format!("Key '{}' not found", key))
        }
    }
}

/// Redis Keys Skill
#[derive(Debug)]
pub struct RedisKeysSkill;

#[async_trait::async_trait]
impl Skill for RedisKeysSkill {
    fn name(&self) -> &str {
        "redis_keys"
    }
    fn description(&self) -> &str {
        "Find keys matching a pattern in Redis"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to list keys in Redis"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::Database
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Redis host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Redis port".to_string(),
                required: false,
                default: Some(Value::Number(6379.into())),
                example: Some(Value::Number(6379.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Redis password".to_string(),
                required: false,
                default: Some(Value::String("".to_string())),
                example: Some(Value::String("".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "db".to_string(),
                param_type: "integer".to_string(),
                description: "Redis database number".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(0.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Key pattern (e.g., 'user:*')".to_string(),
                required: false,
                default: Some(Value::String("*".to_string())),
                example: Some(Value::String("user:*".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "redis_keys", "parameters": { "host": "localhost", "pattern": "user:*" } })
    }

    fn example_output(&self) -> String {
        r#"["user:100", "user:101"]"#.to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 6379) as u16;
        let password = parameters
            .get("password")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let db = get_param_usize(parameters, "db", 0);
        let pattern = parameters
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("*");

        let mut conn = get_redis_connection(&host, port, password, db)?;
        let keys: Vec<String> = conn.keys(pattern)?;
        Ok(json!(keys).to_string())
    }
}

/// Redis Hash Set Skill
#[derive(Debug)]
pub struct RedisHSetSkill;

#[async_trait::async_trait]
impl Skill for RedisHSetSkill {
    fn name(&self) -> &str {
        "redis_hset"
    }
    fn description(&self) -> &str {
        "Set a field in a Redis hash"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to store structured data in Redis"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::Database
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Redis host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Redis port".to_string(),
                required: false,
                default: Some(Value::Number(6379.into())),
                example: Some(Value::Number(6379.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Redis password".to_string(),
                required: false,
                default: Some(Value::String("".to_string())),
                example: Some(Value::String("".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "db".to_string(),
                param_type: "integer".to_string(),
                description: "Redis database number".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(0.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Hash key".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user:100".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "field".to_string(),
                param_type: "string".to_string(),
                description: "Field name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("name".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "value".to_string(),
                param_type: "string".to_string(),
                description: "Value to set".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("John Doe".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "redis_hset", "parameters": { "host": "localhost", "key": "user:100", "field": "name", "value": "John Doe" } })
    }

    fn example_output(&self) -> String {
        "Successfully set field 'name' in hash 'user:100'".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 6379) as u16;
        let password = parameters
            .get("password")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let db = get_param_usize(parameters, "db", 0);
        let key = get_param_string(parameters, "key")?;
        let field = get_param_string(parameters, "field")?;
        let value = get_param_string(parameters, "value")?;

        let mut conn = get_redis_connection(&host, port, password, db)?;
        let _: i32 = conn.hset(&key, &field, &value)?;
        Ok(format!(
            "Successfully set field '{}' in hash '{}'",
            field, key
        ))
    }
}

/// Redis Hash Get Skill
#[derive(Debug)]
pub struct RedisHGetSkill;

#[async_trait::async_trait]
impl Skill for RedisHGetSkill {
    fn name(&self) -> &str {
        "redis_hget"
    }
    fn description(&self) -> &str {
        "Get a field from a Redis hash"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to retrieve structured data from Redis"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::Database
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "Redis host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "Redis port".to_string(),
                required: false,
                default: Some(Value::Number(6379.into())),
                example: Some(Value::Number(6379.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Redis password".to_string(),
                required: false,
                default: Some(Value::String("".to_string())),
                example: Some(Value::String("".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "db".to_string(),
                param_type: "integer".to_string(),
                description: "Redis database number".to_string(),
                required: false,
                default: Some(Value::Number(0.into())),
                example: Some(Value::Number(0.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "key".to_string(),
                param_type: "string".to_string(),
                description: "Hash key".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("user:100".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "field".to_string(),
                param_type: "string".to_string(),
                description: "Field name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("name".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "redis_hget", "parameters": { "host": "localhost", "key": "user:100", "field": "name" } })
    }

    fn example_output(&self) -> String {
        "John Doe".to_string()
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 6379) as u16;
        let password = parameters
            .get("password")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let db = get_param_usize(parameters, "db", 0);
        let key = get_param_string(parameters, "key")?;
        let field = get_param_string(parameters, "field")?;

        let mut conn = get_redis_connection(&host, port, password, db)?;
        let value: Option<String> = conn.hget(&key, &field)?;
        match value {
            Some(v) => Ok(v),
            None => Ok("null".to_string()),
        }
    }
}
