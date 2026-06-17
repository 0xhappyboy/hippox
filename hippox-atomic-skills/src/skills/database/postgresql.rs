use crate::{SkillCallback, SkillContext};
use anyhow::Result;
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Column, Row};
use std::collections::HashMap;
use std::time::Duration;

use crate::SkillCategory;
use crate::types::{Skill, SkillParameter};

async fn get_pg_pool(
    host: &str,
    port: u16,
    database: &str,
    username: &str,
    password: &str,
    pool_size: u32,
    timeout: u64,
) -> Result<PgPool> {
    let url = format!(
        "postgresql://{}:{}@{}:{}/{}",
        username, password, host, port, database
    );
    let pool = PgPoolOptions::new()
        .max_connections(pool_size)
        .acquire_timeout(Duration::from_secs(timeout))
        .connect(&url)
        .await?;
    Ok(pool)
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
    fn category(&self) -> SkillCategory {
        SkillCategory::Database
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "PostgreSQL host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "PostgreSQL port".to_string(),
                required: false,
                default: Some(Value::Number(5432.into())),
                example: Some(Value::Number(5432.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "database".to_string(),
                param_type: "string".to_string(),
                description: "Database name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("myapp".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "Database username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("postgres".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Database password".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("password".to_string())),
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
        json!({ "action": "postgres_query", "parameters": { "host": "localhost", "database": "myapp", "username": "postgres", "password": "password", "query": "SELECT * FROM users WHERE age > $1", "params": [18], "limit": 10 } })
    }

    fn example_output(&self) -> String {
        r#"{"rows": [{"id": 1, "name": "John", "age": 25}], "row_count": 1}"#.to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 5432) as u16;
        let database = get_param_string(parameters, "database")?;
        let username = get_param_string(parameters, "username")?;
        let password = get_param_string(parameters, "password")?;
        let query = get_param_string(parameters, "query")?;
        let limit = get_param_u64(parameters, "limit", 100);

        let default_params = vec![];
        let params = parameters
            .get("params")
            .and_then(|v| v.as_array())
            .unwrap_or(&default_params);

        let pool = get_pg_pool(&host, port, &database, &username, &password, 10, 30).await?;

        let mut query_builder = sqlx::query(&query);
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

        Ok(json!({ "rows": results, "row_count": results.len() }).to_string())
    }
}

/// PostgreSQL Execute Skill
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
    fn category(&self) -> SkillCategory {
        SkillCategory::Database
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "PostgreSQL host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "PostgreSQL port".to_string(),
                required: false,
                default: Some(Value::Number(5432.into())),
                example: Some(Value::Number(5432.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "database".to_string(),
                param_type: "string".to_string(),
                description: "Database name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("myapp".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "Database username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("postgres".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Database password".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("password".to_string())),
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
        json!({ "action": "postgres_execute", "parameters": { "host": "localhost", "database": "myapp", "username": "postgres", "password": "password", "query": "UPDATE users SET age = $1 WHERE name = $2", "params": [26, "John"] } })
    }

    fn example_output(&self) -> String {
        r#"{"rows_affected": 1}"#.to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 5432) as u16;
        let database = get_param_string(parameters, "database")?;
        let username = get_param_string(parameters, "username")?;
        let password = get_param_string(parameters, "password")?;
        let query = get_param_string(parameters, "query")?;

        let default_params = vec![];
        let params = parameters
            .get("params")
            .and_then(|v| v.as_array())
            .unwrap_or(&default_params);

        let pool = get_pg_pool(&host, port, &database, &username, &password, 10, 30).await?;

        let mut query_builder = sqlx::query(&query);
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
        Ok(json!({ "rows_affected": result.rows_affected() }).to_string())
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
    fn category(&self) -> SkillCategory {
        SkillCategory::Database
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "host".to_string(),
                param_type: "string".to_string(),
                description: "PostgreSQL host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "PostgreSQL port".to_string(),
                required: false,
                default: Some(Value::Number(5432.into())),
                example: Some(Value::Number(5432.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "database".to_string(),
                param_type: "string".to_string(),
                description: "Database name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("myapp".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "Database username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("postgres".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Database password".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("password".to_string())),
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
        json!({ "action": "postgres_list_tables", "parameters": { "host": "localhost", "database": "myapp", "username": "postgres", "password": "password", "schema": "public" } })
    }

    fn example_output(&self) -> String {
        r#"["users", "orders", "products"]"#.to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 5432) as u16;
        let database = get_param_string(parameters, "database")?;
        let username = get_param_string(parameters, "username")?;
        let password = get_param_string(parameters, "password")?;
        let schema = parameters
            .get("schema")
            .and_then(|v| v.as_str())
            .unwrap_or("public");

        let pool = get_pg_pool(&host, port, &database, &username, &password, 10, 30).await?;

        let rows = sqlx::query(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = $1 AND table_type = 'BASE TABLE' ORDER BY table_name"
        ).bind(schema).fetch_all(&pool).await?;

        let tables: Vec<String> = rows.iter().map(|row| row.get(0)).collect();
        Ok(json!(tables).to_string())
    }
}
