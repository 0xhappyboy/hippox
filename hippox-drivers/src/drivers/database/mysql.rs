//! MySQL database driver module
//!
//! This module provides drivers for MySQL database operations including
//! query execution, data modification, and table listing.
use crate::DriverCategory;
use crate::types::{Driver, DriverParameter};
use crate::{DriverCallback, DriverContext, DriverError, DriverResult};
use serde_json::{Value, json};
use sqlx::mysql::MySqlPool;
use sqlx::{Column, Row};
use std::collections::HashMap;
use tracing::{debug, info};
/// Creates a MySQL connection pool
///
/// # Arguments
/// * `host` - MySQL server hostname
/// * `port` - MySQL server port
/// * `database` - Database name
/// * `username` - Database username
/// * `password` - Database password
///
/// # Returns
/// * `DriverResult<MySqlPool>` - MySQL connection pool on success
async fn get_mysql_pool(host: &str, port: u16, database: &str, username: &str, password: &str) -> DriverResult<MySqlPool> {
    let url = format!("mysql://{}:{}@{}:{}/{}", username, password, host, port, database);
    debug!("Connecting to MySQL at {}:{}", host, port);
    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&url)
        .await
        .map_err(|e| DriverError::execution(format!("Failed to connect to MySQL: {}", e)))?;
    info!("Successfully connected to MySQL database: {}", database);
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
/// Driver for executing SELECT queries on MySQL
#[derive(Debug)]
pub struct MysqlQueryDriver;
#[async_trait::async_trait]
impl Driver for MysqlQueryDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "mysql_query";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Execute SELECT query on MySQL database";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to query data from MySQL database";
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
                description: "MySQL host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "MySQL port".to_string(),
                required: false,
                default: Some(Value::Number(3306.into())),
                example: Some(Value::Number(3306.into())),
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
                example: Some(Value::String("root".to_string())),
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
                description: "SQL SELECT query to execute".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("SELECT * FROM users WHERE age > ?".to_string())),
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
            "action": "mysql_query",
            "parameters": {
                "host": "localhost",
                "database": "myapp",
                "username": "root",
                "password": "password",
                "query": "SELECT * FROM users WHERE age > ?",
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
        debug!("Executing mysql_query driver");
        // Extract required parameters
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 3306) as u16;
        let database = get_param_string(parameters, "database")?;
        let username = get_param_string(parameters, "username")?;
        let password = get_param_string(parameters, "password")?;
        let query = get_param_string(parameters, "query")?;
        let limit = get_param_u64(parameters, "limit", 100);
        // Extract query parameters
        let default_params = vec![];
        let params = parameters.get("params").and_then(|v| v.as_array()).unwrap_or(&default_params);
        debug!("Connecting to MySQL database: {}", database);
        let pool = get_mysql_pool(&host, port, &database, &username, &password).await?;
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
                                    let bytes_val: Result<Vec<u8>, sqlx::Error> = row.try_get(idx);
                                    if let Ok(val) = bytes_val {
                                        row_map.insert(column_name.to_string(), json!(format!("{:?}", val)));
                                    } else {
                                        row_map.insert(column_name.to_string(), json!(null));
                                    }
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
/// Driver for executing INSERT, UPDATE, or DELETE queries on MySQL
#[derive(Debug)]
pub struct MysqlExecuteDriver;
#[async_trait::async_trait]
impl Driver for MysqlExecuteDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "mysql_execute";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Execute INSERT, UPDATE, or DELETE query on MySQL database";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user needs to modify data in MySQL database";
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
                description: "MySQL host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "MySQL port".to_string(),
                required: false,
                default: Some(Value::Number(3306.into())),
                example: Some(Value::Number(3306.into())),
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
                example: Some(Value::String("root".to_string())),
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
                example: Some(Value::String("INSERT INTO users (name, age) VALUES (?, ?)".to_string())),
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
            "action": "mysql_execute",
            "parameters": {
                "host": "localhost",
                "database": "myapp",
                "username": "root",
                "password": "password",
                "query": "UPDATE users SET age = ? WHERE name = ?",
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
        debug!("Executing mysql_execute driver");
        // Extract required parameters
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 3306) as u16;
        let database = get_param_string(parameters, "database")?;
        let username = get_param_string(parameters, "username")?;
        let password = get_param_string(parameters, "password")?;
        let query = get_param_string(parameters, "query")?;
        // Extract query parameters
        let default_params = vec![];
        let params = parameters.get("params").and_then(|v| v.as_array()).unwrap_or(&default_params);
        debug!("Connecting to MySQL database: {}", database);
        let pool = get_mysql_pool(&host, port, &database, &username, &password).await?;
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
/// Driver for listing all tables in a MySQL database
#[derive(Debug)]
pub struct MysqlListTablesDriver;
#[async_trait::async_trait]
impl Driver for MysqlListTablesDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "mysql_list_tables";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "List all tables in MySQL database";
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
                description: "MySQL host".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("localhost".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "port".to_string(),
                param_type: "integer".to_string(),
                description: "MySQL port".to_string(),
                required: false,
                default: Some(Value::Number(3306.into())),
                example: Some(Value::Number(3306.into())),
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
                example: Some(Value::String("root".to_string())),
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
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "mysql_list_tables",
            "parameters": {
                "host": "localhost",
                "database": "myapp",
                "username": "root",
                "password": "password"
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
        debug!("Executing mysql_list_tables driver");
        // Extract required parameters
        let host = get_param_string(parameters, "host")?;
        let port = get_param_u64(parameters, "port", 3306) as u16;
        let database = get_param_string(parameters, "database")?;
        let username = get_param_string(parameters, "username")?;
        let password = get_param_string(parameters, "password")?;
        debug!("Connecting to MySQL database: {}", database);
        let pool = get_mysql_pool(&host, port, &database, &username, &password).await?;
        debug!("Fetching table list");
        let rows = sqlx::query("SHOW TABLES").fetch_all(&pool).await.map_err(|e| DriverError::execution(format!("Failed to list tables: {}", e)))?;
        let tables: Vec<String> = rows.iter().map(|row| row.get(0)).collect();
        info!("Found {} tables in database {}", tables.len(), database);
        return Ok(json!(tables).to_string());
    }
}
