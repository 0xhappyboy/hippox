use anyhow::Result;
use serde_json::{Value, json};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::{Column, Row};
use std::collections::HashMap;

use crate::{SkillCallback, SkillCategory, SkillContext};
use crate::types::{Skill, SkillParameter};

async fn get_sqlite_pool(path: &str, pool_size: u32) -> Result<SqlitePool> {
    let url = format!("sqlite:{}", path);
    let pool = SqlitePoolOptions::new()
        .max_connections(pool_size)
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

/// SQLite Query Skill
#[derive(Debug)]
pub struct SqliteQuerySkill;

#[async_trait::async_trait]
impl Skill for SqliteQuerySkill {
    fn name(&self) -> &str {
        "sqlite_query"
    }
    fn description(&self) -> &str {
        "Execute SELECT query on SQLite database"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to query data from SQLite database"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::Database
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "SQLite database file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/database.db".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "SQL SELECT query to execute".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "SELECT * FROM users WHERE age > ?".to_string(),
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
        json!({ "action": "sqlite_query", "parameters": { "path": "/path/to/database.db", "query": "SELECT * FROM users WHERE age > ?", "params": [18], "limit": 10 } })
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
        let path = get_param_string(parameters, "path")?;
        let query = get_param_string(parameters, "query")?;
        let limit = get_param_u64(parameters, "limit", 100);
        let default_params = vec![];
        let params = parameters
            .get("params")
            .and_then(|v| v.as_array())
            .unwrap_or(&default_params);

        let pool = get_sqlite_pool(&path, 5).await?;

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
                let value: Result<String, sqlx::Error> = row.try_get(idx);
                if let Ok(val) = value {
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
            results.push(serde_json::Value::Object(row_map));
        }

        Ok(json!({ "rows": results, "row_count": results.len() }).to_string())
    }
}

/// SQLite Execute Skill
#[derive(Debug)]
pub struct SqliteExecuteSkill;

#[async_trait::async_trait]
impl Skill for SqliteExecuteSkill {
    fn name(&self) -> &str {
        "sqlite_execute"
    }
    fn description(&self) -> &str {
        "Execute INSERT, UPDATE, or DELETE query on SQLite database"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to modify data in SQLite database"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::Database
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "SQLite database file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("/path/to/database.db".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "SQL query to execute (INSERT, UPDATE, DELETE)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String(
                    "INSERT INTO users (name, age) VALUES (?, ?)".to_string(),
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
        json!({ "action": "sqlite_execute", "parameters": { "path": "/path/to/database.db", "query": "UPDATE users SET age = ? WHERE name = ?", "params": [26, "John"] } })
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
        let path = get_param_string(parameters, "path")?;
        let query = get_param_string(parameters, "query")?;
        let default_params = vec![];
        let params = parameters
            .get("params")
            .and_then(|v| v.as_array())
            .unwrap_or(&default_params);

        let pool = get_sqlite_pool(&path, 5).await?;

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

/// SQLite List Tables Skill
#[derive(Debug)]
pub struct SqliteListTablesSkill;

#[async_trait::async_trait]
impl Skill for SqliteListTablesSkill {
    fn name(&self) -> &str {
        "sqlite_list_tables"
    }
    fn description(&self) -> &str {
        "List all tables in SQLite database"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to see available tables in the SQLite database"
    }
    fn category(&self) -> SkillCategory {
        SkillCategory::Database
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "SQLite database file path".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("/path/to/database.db".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "sqlite_list_tables", "parameters": { "path": "/path/to/database.db" } })
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
        let path = get_param_string(parameters, "path")?;
        let pool = get_sqlite_pool(&path, 5).await?;
        let rows = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .fetch_all(&pool)
            .await?;
        let tables: Vec<String> = rows.iter().map(|row| row.get(0)).collect();
        Ok(json!(tables).to_string())
    }
}
