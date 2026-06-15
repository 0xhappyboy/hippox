use crate::config::get_config;
use crate::config::get_redis_instance;
use crate::executors::types::{Skill, SkillParameter};
use anyhow::Result;
use once_cell::sync::Lazy;
use redis::Client;
use redis::Commands;
use redis::Connection;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct RedisConnectionPool {
    client: Arc<Mutex<Option<Client>>>,
    instance_id: String,
}

impl RedisConnectionPool {
    fn new(instance_id: String) -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            instance_id,
        }
    }

    async fn get_client(&self) -> Result<Client> {
        let mut client_guard = self.client.lock().await;
        if let Some(client) = client_guard.as_ref() {
            return Ok(client.clone());
        }
        let instance = get_redis_instance(&self.instance_id)
            .ok_or_else(|| anyhow::anyhow!("Redis instance '{}' not found", self.instance_id))?;

        let redis_url = if instance.password.is_empty() {
            format!("redis://{}:{}/", instance.host, instance.port)
        } else {
            format!(
                "redis://:{}@{}:{}/{}",
                instance.password, instance.host, instance.port, instance.db
            )
        };
        let client = Client::open(redis_url)?;
        *client_guard = Some(client.clone());
        Ok(client)
    }

    async fn get_connection(&self) -> Result<Connection> {
        let client = self.get_client().await?;
        let conn = client.get_connection()?;
        Ok(conn)
    }
}

static REDIS_POOLS: Lazy<Arc<Mutex<HashMap<String, RedisConnectionPool>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

async fn get_redis_pool(instance_id: &str) -> Result<RedisConnectionPool> {
    let mut pools = REDIS_POOLS.lock().await;
    if !pools.contains_key(instance_id) {
        pools.insert(
            instance_id.to_string(),
            RedisConnectionPool::new(instance_id.to_string()),
        );
    }
    Ok(pools.get(instance_id).unwrap().clone())
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

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "Redis instance ID (from config)".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(Value::String("redis_cache".to_string())),
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
                description: "Time to live in seconds (optional)".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(3600.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "redis_set",
            "parameters": {
                "instance_id": "redis_cache",
                "key": "user:100",
                "value": "John Doe",
                "ttl": 3600
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully set key 'user:100'".to_string()
    }

    fn category(&self) -> &str {
        "database"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters
            .get("instance_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let pool = get_redis_pool(instance_id).await?;
        let mut conn = pool.get_connection().await?;

        let key = parameters
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: key"))?;
        let value = parameters
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: value"))?;
        let _: () = conn.set(key, value)?;
        if let Some(ttl) = parameters.get("ttl").and_then(|v| v.as_u64()) {
            let _: () = conn.expire(key, ttl.try_into().unwrap())?;
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

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "Redis instance ID (from config)".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(Value::String("redis_cache".to_string())),
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
        json!({
            "action": "redis_get",
            "parameters": {
                "instance_id": "redis_cache",
                "key": "user:100"
            }
        })
    }

    fn example_output(&self) -> String {
        "John Doe".to_string()
    }

    fn category(&self) -> &str {
        "database"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters
            .get("instance_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let pool = get_redis_pool(instance_id).await?;
        let mut conn = pool.get_connection().await?;

        let key = parameters
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: key"))?;
        let value: Option<String> = conn.get(key)?;
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

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "Redis instance ID (from config)".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(Value::String("redis_cache".to_string())),
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
        json!({
            "action": "redis_del",
            "parameters": {
                "instance_id": "redis_cache",
                "key": "user:100"
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully deleted key 'user:100'".to_string()
    }

    fn category(&self) -> &str {
        "database"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters
            .get("instance_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let pool = get_redis_pool(instance_id).await?;
        let mut conn = pool.get_connection().await?;

        let key = parameters
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: key"))?;
        let deleted: i32 = conn.del(key)?;
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

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "Redis instance ID (from config)".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(Value::String("redis_cache".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "pattern".to_string(),
                param_type: "string".to_string(),
                description: "Key pattern (e.g., user:*)".to_string(),
                required: false,
                default: Some(Value::String("*".to_string())),
                example: Some(Value::String("user:*".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "redis_keys",
            "parameters": {
                "instance_id": "redis_cache",
                "pattern": "user:*"
            }
        })
    }

    fn example_output(&self) -> String {
        r#"["user:100", "user:101", "user:102"]"#.to_string()
    }

    fn category(&self) -> &str {
        "database"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters
            .get("instance_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let pool = get_redis_pool(instance_id).await?;
        let mut conn = pool.get_connection().await?;

        let pattern = parameters
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("*");
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

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "Redis instance ID (from config)".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(Value::String("redis_cache".to_string())),
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
        json!({
            "action": "redis_hset",
            "parameters": {
                "instance_id": "redis_cache",
                "key": "user:100",
                "field": "name",
                "value": "John Doe"
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully set field 'name' in hash 'user:100'".to_string()
    }

    fn category(&self) -> &str {
        "database"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters
            .get("instance_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let pool = get_redis_pool(instance_id).await?;
        let mut conn = pool.get_connection().await?;

        let key = parameters
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: key"))?;
        let field = parameters
            .get("field")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: field"))?;
        let value = parameters
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: value"))?;
        let _: i32 = conn.hset(key, field, value)?;
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

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "Redis instance ID (from config)".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(Value::String("redis_cache".to_string())),
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
        json!({
            "action": "redis_hget",
            "parameters": {
                "instance_id": "redis_cache",
                "key": "user:100",
                "field": "name"
            }
        })
    }

    fn example_output(&self) -> String {
        "John Doe".to_string()
    }

    fn category(&self) -> &str {
        "database"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters
            .get("instance_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let pool = get_redis_pool(instance_id).await?;
        let mut conn = pool.get_connection().await?;

        let key = parameters
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: key"))?;
        let field = parameters
            .get("field")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: field"))?;
        let value: Option<String> = conn.hget(key, field)?;
        match value {
            Some(v) => Ok(v),
            None => Ok("null".to_string()),
        }
    }
}
