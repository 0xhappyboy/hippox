//! PostgreSQL database driver module
//!
//! This module provides drivers for PostgreSQL database operations including
//! query execution, data modification, and table listing.
use crate::DriverCategory;
use crate::types::{Driver, DriverParameter};
use crate::{DriverCallback, DriverContext, DriverError, DriverResult};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Column, Row};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};
/// Creates a PostgreSQL connection pool
///
/// # Arguments
/// * `host` - PostgreSQL server hostname
/// * `port` - PostgreSQL server port
/// * `database` - Database name
/// * `username` - Database username
/// * `password` - Database password
/// * `pool_size` - Maximum number of connections in the pool
/// * `timeout` - Connection acquisition timeout in seconds
///
/// # Returns
/// * `DriverResult<PgPool>` - PostgreSQL connection pool on success
async fn get_pg_pool(host: &str, port: u16, database: &str, username: &str, password: &str, pool_size: u32, timeout: u64) -> DriverResult<PgPool> {
    let url = format!("postgresql://{}:{}@{}:{}/{}", username, password, host, port, database);
    debug!("Connecting to PostgreSQL at {}:{}", host, port);
    let pool = PgPoolOptions::new()
        .max_connections(pool_size)
        .acquire_timeout(Duration::from_secs(timeout))
        .connect(&url)
        .await
        .map_err(|e| DriverError::execution(format!("Failed to connect to PostgreSQL: {}", e)))?;
    info!("Successfully connected to PostgreSQL database: {}", database);
    return Ok(pool);
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
/// Driver for executing SELECT queries on PostgreSQL
#[derive(Debug)]
pub struct PostgresQueryDriver;
#[async_trait::async_trait]
impl Driver for PostgresQueryDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "postgres_query";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Execute SELECT query on PostgreSQL database";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to query data from PostgreSQL database";
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
                description: "PostgreSQL host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "PostgreSQL port".to_string(),
                required: false,
                default: Some(Value::Number(5432.into())),
                example: Some(Value::Number(5432.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "database".to_string(),
                param_type: "string".to_string(),
                description: "Database name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("myapp".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "Database username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("postgres".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Database password".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("password".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "SQL SELECT query to execute (use $1, $2 for parameters)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("SELECT * FROM users WHERE age > $1".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "params".to_string(),
                param_type: "array".to_string(),
                description: "Query parameters".to_string(),
                required: false,
                default: Some(Value::Array(vec![])),
                example: Some(json!([18])),
                enum_values: None,
            },
            DriverParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of rows to return".to_string(),
                required: false,
                default: Some(Value::Number(100.into())),
                example: Some(Value::Number(50.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "postgres_query",
            "parameters": {
                "host": "localhost",
                "database": "myapp",
                "username": "postgres",
                "password": "password",
                "query": "SELECT * FROM users WHERE age > $1",
                "params": [18],
                "limit": 10
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"{"rows": [{"id": 1, "name": "John", "age": 25}], "row_count": 1}"#.to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing postgres_query driver");
        // Extract required parameters
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 5432) as u16;
        let database = get_param_string(parameters, "database")?;
        let username = get_param_string(parameters, "username")?;
        let password = get_param_string(parameters, "password")?;
        let query = get_param_string(parameters, "query")?;
        let limit = get_param_u64(parameters, "limit", 100);
        // Extract query parameters
        let default_params = vec![];
        let params = parameters.get("params").and_then(|v| v.as_array()).unwrap_or(&default_params);
        debug!("Connecting to PostgreSQL database: {}", database);
        let pool = get_pg_pool(&host, port, &database, &username, &password, 10, 30).await?;
        // Build the query with parameters
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
        debug!("Executing query: {}", query);
        let rows = query_builder.fetch_all(&pool).await.map_err(|e| DriverError::execution(format!("Query execution failed: {}", e)))?;
        // Convert rows to JSON
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
        info!("Query returned {} rows", results.len());
        return Ok(json!({ "rows": results, "row_count": results.len() }).to_string());
    }
}
/// Driver for executing INSERT, UPDATE, or DELETE queries on PostgreSQL
#[derive(Debug)]
pub struct PostgresExecuteDriver;
#[async_trait::async_trait]
impl Driver for PostgresExecuteDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "postgres_execute";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Execute INSERT, UPDATE, or DELETE query on PostgreSQL database";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to modify data in PostgreSQL database";
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
                description: "PostgreSQL host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "PostgreSQL port".to_string(),
                required: false,
                default: Some(Value::Number(5432.into())),
                example: Some(Value::Number(5432.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "database".to_string(),
                param_type: "string".to_string(),
                description: "Database name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("myapp".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "Database username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("postgres".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Database password".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("password".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "SQL query to execute (INSERT, UPDATE, DELETE)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("INSERT INTO users (name, age) VALUES ($1, $2)".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "params".to_string(),
                param_type: "array".to_string(),
                description: "Query parameters".to_string(),
                required: false,
                default: Some(Value::Array(vec![])),
                example: Some(json!(["John", 25])),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "postgres_execute",
            "parameters": {
                "host": "localhost",
                "database": "myapp",
                "username": "postgres",
                "password": "password",
                "query": "UPDATE users SET age = $1 WHERE name = $2",
                "params": [26, "John"]
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"{"rows_affected": 1}"#.to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing postgres_execute driver");
        // Extract required parameters
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 5432) as u16;
        let database = get_param_string(parameters, "database")?;
        let username = get_param_string(parameters, "username")?;
        let password = get_param_string(parameters, "password")?;
        let query = get_param_string(parameters, "query")?;
        // Extract query parameters
        let default_params = vec![];
        let params = parameters.get("params").and_then(|v| v.as_array()).unwrap_or(&default_params);
        debug!("Connecting to PostgreSQL database: {}", database);
        let pool = get_pg_pool(&host, port, &database, &username, &password, 10, 30).await?;
        // Build the query with parameters
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
        debug!("Executing query: {}", query);
        let result = query_builder.execute(&pool).await.map_err(|e| DriverError::execution(format!("Query execution failed: {}", e)))?;
        info!("Query affected {} rows", result.rows_affected());
        return Ok(json!({ "rows_affected": result.rows_affected() }).to_string());
    }
}
/// Driver for listing all tables in a PostgreSQL database
#[derive(Debug)]
pub struct PostgresListTablesDriver;
#[async_trait::async_trait]
impl Driver for PostgresListTablesDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "postgres_list_tables";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "List all tables in PostgreSQL database";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to see available tables in the database";
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
                description: "PostgreSQL host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "PostgreSQL port".to_string(),
                required: false,
                default: Some(Value::Number(5432.into())),
                example: Some(Value::Number(5432.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "database".to_string(),
                param_type: "string".to_string(),
                description: "Database name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("myapp".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "Database username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("postgres".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "password".to_string(),
                param_type: "string".to_string(),
                description: "Database password".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("password".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "schema".to_string(),
                param_type: "string".to_string(),
                description: "Schema name (default: public)".to_string(),
                required: false,
                default: Some(Value::String("public".to_string())),
                example: Some(Value::String("public".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "postgres_list_tables",
            "parameters": {
                "host": "localhost",
                "database": "myapp",
                "username": "postgres",
                "password": "password",
                "schema": "public"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return r#"["users", "orders", "products"]"#.to_string();
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing postgres_list_tables driver");
        // Extract required parameters
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 5432) as u16;
        let database = get_param_string(parameters, "database")?;
        let username = get_param_string(parameters, "username")?;
        let password = get_param_string(parameters, "password")?;
        let schema = parameters.get("schema").and_then(|v| v.as_str()).unwrap_or("public");
        debug!("Connecting to PostgreSQL database: {}", database);
        let pool = get_pg_pool(&host, port, &database, &username, &password, 10, 30).await?;
        debug!("Fetching table list from schema: {}", schema);
        let rows =
            sqlx::query("SELECT table_name FROM information_schema.tables WHERE table_schema = $1 AND table_type = 'BASE TABLE' ORDER BY table_name")
                .bind(schema)
                .fetch_all(&pool)
                .await
                .map_err(|e| DriverError::execution(format!("Failed to list tables: {}", e)))?;
        let tables: Vec<String> = rows.iter().map(|row| row.get(0)).collect();
        info!("Found {} tables in database {}", tables.len(), database);
        return Ok(json!(tables).to_string());
    }
}
