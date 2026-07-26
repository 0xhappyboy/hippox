//! CSV file driver module
//!
//! This module provides drivers for CSV file operations including
//! reading CSV files, parsing tabular data, and writing CSV files.
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory, DriverError, DriverResult, ensure_dir, file_exists, read_file_content,
    types::{Driver, DriverParameter},
    validate_path, write_file_content,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for reading and parsing CSV files
#[derive(Debug)]
pub struct CsvReadDriver;
#[async_trait::async_trait]
impl Driver for CsvReadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "csv_read";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Read and parse CSV file content into structured data";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to read a CSV file, analyze tabular data, or extract information from spreadsheets";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the CSV file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("data.csv".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "has_header".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether the CSV has a header row".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            DriverParameter {
                name: "delimiter".to_string(),
                param_type: "string".to_string(),
                description: "CSV delimiter character (default: ',')".to_string(),
                required: false,
                default: Some(Value::String(",".to_string())),
                example: Some(Value::String(";".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of rows to read".to_string(),
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
            "action": "csv_read",
            "parameters": {
                "path": "data.csv"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Header: [name, age, city]\nRow 1: [Alice, 30, Beijing]\nRow 2: [Bob, 25, Shanghai]".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Document;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing csv_read driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let has_header = parameters.get("has_header").and_then(|v| v.as_bool()).unwrap_or(true);
        let delimiter = parameters.get("delimiter").and_then(|v| v.as_str()).unwrap_or(",").chars().next().unwrap_or(',');
        let limit = parameters.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
        debug!("Reading CSV file: {}, limit: {}", path, limit);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("CSV file not found: {}", path)));
        }
        let content =
            read_file_content(&validated_path.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to read file: {}", e)))?;
        debug!("Parsing CSV content with delimiter: {}", delimiter);
        let mut reader = csv::ReaderBuilder::new().has_headers(has_header).delimiter(delimiter as u8).from_reader(content.as_bytes());
        let headers: Vec<String> = if has_header {
            reader
                .headers()
                .map_err(|e| DriverError::execution(format!("Failed to read CSV headers: {}", e)))?
                .iter()
                .map(|h| h.to_string())
                .collect()
        } else {
            (0..reader.headers().map_err(|e| DriverError::execution(format!("Failed to read CSV: {}", e)))?.len())
                .map(|i| format!("Column_{}", i + 1))
                .collect()
        };
        let mut rows = Vec::new();
        for (i, result) in reader.records().enumerate() {
            if i >= limit {
                let remaining = reader.records().count();
                rows.push(format!("... and {} more rows", remaining));
                break;
            }
            let record = result.map_err(|e| DriverError::execution(format!("Failed to read CSV row: {}", e)))?;
            let row: Vec<String> = record.iter().map(|f| f.to_string()).collect();
            rows.push(format!("{:?}", row));
        }
        let mut output = format!("Headers: {:?}\n", headers);
        for (i, row) in rows.iter().enumerate() {
            output.push_str(&format!("Row {}: {}\n", i + 1, row));
        }
        info!("CSV read completed, found {} headers and {} rows", headers.len(), rows.len());
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Driver for writing CSV files
#[derive(Debug)]
pub struct CsvWriteDriver;
#[async_trait::async_trait]
impl Driver for CsvWriteDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "csv_write";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Write structured data to a CSV file";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to save tabular data, export to CSV, or create a spreadsheet";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the CSV file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("output.csv".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "headers".to_string(),
                param_type: "array".to_string(),
                description: "Column headers as an array of strings".to_string(),
                required: true,
                default: None,
                example: Some(json!(["name", "age", "city"])),
                enum_values: None,
            },
            DriverParameter {
                name: "rows".to_string(),
                param_type: "array".to_string(),
                description: "Data rows as array of arrays".to_string(),
                required: true,
                default: None,
                example: Some(json!([["Alice", "30", "Beijing"], ["Bob", "25", "Shanghai"]])),
                enum_values: None,
            },
            DriverParameter {
                name: "delimiter".to_string(),
                param_type: "string".to_string(),
                description: "CSV delimiter character (default: ',')".to_string(),
                required: false,
                default: Some(Value::String(",".to_string())),
                example: Some(Value::String(";".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "csv_write",
            "parameters": {
                "path": "output.csv",
                "headers": ["name", "age"],
                "rows": [["Alice", "30"], ["Bob", "25"]]
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "CSV written to: output.csv (2 rows)".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Document;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing csv_write driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let headers_json = parameters.get("headers").ok_or_else(|| DriverError::missing_parameter("headers"))?;
        let rows_json = parameters.get("rows").ok_or_else(|| DriverError::missing_parameter("rows"))?;
        let headers = headers_json
            .as_array()
            .ok_or_else(|| DriverError::invalid_type("headers", "array", "other"))?
            .iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .collect::<Vec<_>>();
        let rows = rows_json.as_array().ok_or_else(|| DriverError::invalid_type("rows", "array", "other"))?;
        let delimiter = parameters.get("delimiter").and_then(|v| v.as_str()).unwrap_or(",").chars().next().unwrap_or(',');
        debug!("Writing CSV file: {} with {} headers and {} rows", path, headers.len(), rows.len());
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to create directory: {}", e)))?;
        }
        let mut csv_content = String::new();
        csv_content.push_str(&headers.join(&delimiter.to_string()));
        csv_content.push('\n');
        for row in rows {
            let row_array = row.as_array().ok_or_else(|| DriverError::execution("Each row must be an array".to_string()))?;
            let row_str: Vec<String> = row_array.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect();
            csv_content.push_str(&row_str.join(&delimiter.to_string()));
            csv_content.push('\n');
        }
        write_file_content(&validated_path.to_string_lossy(), &csv_content, false)
            .map_err(|e| DriverError::execution(format!("Failed to write file: {}", e)))?;
        info!("CSV written to: {} ({} rows)", path, rows.len());
        return Ok(format!("CSV written to: {} ({} rows)", path, rows.len()));
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        parameters.get("headers").ok_or_else(|| DriverError::missing_parameter("headers"))?;
        parameters.get("rows").ok_or_else(|| DriverError::missing_parameter("rows"))?;
        return Ok(());
    }
}
