use crate::config::get_config;
use crate::config::get_postgresql_instance;
use crate::executors::types::{Skill, SkillParameter};
use anyhow::Result;
use once_cell::sync::Lazy;
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Column, Row};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

struct PgConnectionPool {
    pool: Arc<Mutex<Option<PgPool>>>,
    instance_id: String,
}

impl PgConnectionPool {
    fn new(instance_id: String) -> Self {
        Self {
            pool: Arc::new(Mutex::new(None)),
            instance_id,
        }
    }

    async fn get_pool(&self) -> Result<PgPool> {
        let mut pool_guard = self.pool.lock().await;
        if let Some(pool) = pool_guard.as_ref() {
            return Ok(pool.clone());
        }
        let instance = get_postgresql_instance(&self.instance_id).ok_or_else(|| {
            anyhow::anyhow!("PostgreSQL instance '{}' not found", self.instance_id)
        })?;

        let url = format!(
            "postgresql://{}:{}@{}:{}/{}",
            instance.username, instance.password, instance.host, instance.port, instance.database
        );
        let pool = PgPoolOptions::new()
            .max_connections(instance.pool_size as u32)
            .acquire_timeout(Duration::from_secs(instance.timeout))
            .connect(&url)
            .await?;
        *pool_guard = Some(pool.clone());
        Ok(pool)
    }
}

static PG_POOLS: Lazy<Arc<Mutex<HashMap<String, PgConnectionPool>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

async fn get_pg_pool(instance_id: &str) -> Result<PgPool> {
    let mut pools = PG_POOLS.lock().await;
    if !pools.contains_key(instance_id) {
        pools.insert(
            instance_id.to_string(),
            PgConnectionPool::new(instance_id.to_string()),
        );
    }
    let pool = pools.get(instance_id).unwrap();
    pool.get_pool().await
}

/// PostgreSQL Query Skill
#[derive(Debug)]
pub struct PostgresQuerySkill;

#[async_trait::async_trait]
impl Skill for PostgresQuerySkill {
    fn name(&self) -> &str {
        "postgres_query"
    }

    fn description(&self) -> &str {
        "Execute SELECT query on PostgreSQL database"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to query data from PostgreSQL database"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "PostgreSQL instance ID (from config)".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(Value::String("pg_prod".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "SQL SELECT query to execute (use $1, $2 for parameters)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "SELECT * FROM users WHERE age > $1".to_string(),
                )),
                enum_values: None,
            },
            SkillParameter {
                name: "params".to_string(),
                param_type: "array".to_string(),
                description: "Query parameters".to_string(),
                required: false,
                default: Some(Value::Array(vec![])),
                example: Some(json!([18])),
                enum_values: None,
            },
            SkillParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of rows to return".to_string(),
                required: false,
                default: Some(Value::Number(100.into())),
                example: Some(Value::Number(50.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "postgres_query",
            "parameters": {
                "instance_id": "pg_prod",
                "query": "SELECT * FROM users WHERE age > $1",
                "params": [18],
                "limit": 10
            }
        })
    }

    fn example_output(&self) -> String {
        r#"{"rows": [{"id": 1, "name": "John", "age": 25}], "row_count": 1}"#.to_string()
    }

    fn category(&self) -> &str {
        "database"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters
            .get("instance_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let pool = get_pg_pool(instance_id).await?;

        let query = parameters
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;
        let default_params = vec![];
        let params = parameters
            .get("params")
            .and_then(|v| v.as_array())
            .unwrap_or(&default_params);
        let limit = parameters
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(100);
        let mut query_builder = sqlx::query(query);
        for param in params {
            query_builder = match param {
                Value::String(s) => query_builder.bind(s),
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        query_builder.bind(i)
                    } else if let Some(u) = n.as_u64() {
                        query_builder.bind(u as i64)
                    } else if let Some(f) = n.as_f64() {
                        query_builder.bind(f)
                    } else {
                        query_builder.bind(param.to_string())
                    }
                }
                Value::Bool(b) => query_builder.bind(*b),
                Value::Null => query_builder.bind(None::<String>),
                _ => query_builder.bind(param.to_string()),
            };
        }
        let rows = query_builder.fetch_all(&pool).await?;
        let mut results = Vec::new();
        for row in rows.iter().take(limit as usize) {
            let mut row_map = serde_json::Map::new();
            let columns = row.columns();
            for (idx, column) in columns.into_iter().enumerate() {
                let column_name = column.name();
                let json_value: Result<serde_json::Value, sqlx::Error> = row.try_get(idx);
                if let Ok(val) = json_value {
                    row_map.insert(column_name.to_string(), val);
                } else {
                    let str_val: Result<String, sqlx::Error> = row.try_get(idx);
                    if let Ok(val) = str_val {
                        row_map.insert(column_name.to_string(), json!(val));
                    } else {
                        let int_val: Result<i64, sqlx::Error> = row.try_get(idx);
                        if let Ok(val) = int_val {
                            row_map.insert(column_name.to_string(), json!(val));
                        } else {
                            let float_val: Result<f64, sqlx::Error> = row.try_get(idx);
                            if let Ok(val) = float_val {
                                row_map.insert(column_name.to_string(), json!(val));
                            } else {
                                let bool_val: Result<bool, sqlx::Error> = row.try_get(idx);
                                if let Ok(val) = bool_val {
                                    row_map.insert(column_name.to_string(), json!(val));
                                } else {
                                    row_map.insert(column_name.to_string(), json!(null));
                                }
                            }
                        }
                    }
                }
            }
            results.push(serde_json::Value::Object(row_map));
        }
        Ok(json!({
            "rows": results,
            "row_count": results.len()
        })
        .to_string())
    }
}

/// PostgreSQL Execute Skill (INSERT, UPDATE, DELETE)
#[derive(Debug)]
pub struct PostgresExecuteSkill;

#[async_trait::async_trait]
impl Skill for PostgresExecuteSkill {
    fn name(&self) -> &str {
        "postgres_execute"
    }

    fn description(&self) -> &str {
        "Execute INSERT, UPDATE, or DELETE query on PostgreSQL database"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to modify data in PostgreSQL database"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "PostgreSQL instance ID (from config)".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(Value::String("pg_prod".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "SQL query to execute (INSERT, UPDATE, DELETE)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "INSERT INTO users (name, age) VALUES ($1, $2)".to_string(),
                )),
                enum_values: None,
            },
            SkillParameter {
                name: "params".to_string(),
                param_type: "array".to_string(),
                description: "Query parameters".to_string(),
                required: false,
                default: Some(Value::Array(vec![])),
                example: Some(json!(["John", 25])),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "postgres_execute",
            "parameters": {
                "instance_id": "pg_prod",
                "query": "UPDATE users SET age = $1 WHERE name = $2",
                "params": [26, "John"]
            }
        })
    }

    fn example_output(&self) -> String {
        r#"{"rows_affected": 1}"#.to_string()
    }

    fn category(&self) -> &str {
        "database"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters
            .get("instance_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let pool = get_pg_pool(instance_id).await?;

        let query = parameters
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;
        let default_params = vec![];
        let params = parameters
            .get("params")
            .and_then(|v| v.as_array())
            .unwrap_or(&default_params);
        let mut query_builder = sqlx::query(query);
        for param in params {
            query_builder = match param {
                Value::String(s) => query_builder.bind(s),
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        query_builder.bind(i)
                    } else if let Some(u) = n.as_u64() {
                        query_builder.bind(u as i64)
                    } else if let Some(f) = n.as_f64() {
                        query_builder.bind(f)
                    } else {
                        query_builder.bind(param.to_string())
                    }
                }
                Value::Bool(b) => query_builder.bind(*b),
                Value::Null => query_builder.bind(None::<String>),
                _ => query_builder.bind(param.to_string()),
            };
        }
        let result = query_builder.execute(&pool).await?;
        Ok(json!({
            "rows_affected": result.rows_affected()
        })
        .to_string())
    }
}

/// PostgreSQL List Tables Skill
#[derive(Debug)]
pub struct PostgresListTablesSkill;

#[async_trait::async_trait]
impl Skill for PostgresListTablesSkill {
    fn name(&self) -> &str {
        "postgres_list_tables"
    }

    fn description(&self) -> &str {
        "List all tables in PostgreSQL database"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to see available tables in the database"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description: "PostgreSQL instance ID (from config)".to_string(),
                required: false,
                default: Some(Value::String("default".to_string())),
                example: Some(Value::String("pg_prod".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "schema".to_string(),
                param_type: "string".to_string(),
                description: "Schema name (default: public)".to_string(),
                required: false,
                default: Some(Value::String("public".to_string())),
                example: Some(Value::String("public".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "postgres_list_tables",
            "parameters": {
                "instance_id": "pg_prod",
                "schema": "public"
            }
        })
    }

    fn example_output(&self) -> String {
        r#"["users", "orders", "products"]"#.to_string()
    }

    fn category(&self) -> &str {
        "database"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters
            .get("instance_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let pool = get_pg_pool(instance_id).await?;

        let schema = parameters
            .get("schema")
            .and_then(|v| v.as_str())
            .unwrap_or("public");
        let rows = sqlx::query(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = $1 AND table_type = 'BASE TABLE' ORDER BY table_name"
        )
        .bind(schema)
        .fetch_all(&pool)
        .await?;
        let tables: Vec<String> = rows.iter().map(|row| row.get(0)).collect();
        Ok(json!(tables).to_string())
    }
}
