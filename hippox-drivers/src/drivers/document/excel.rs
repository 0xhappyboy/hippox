//! Excel file driver module
//!
//! This module provides drivers for Microsoft Excel XLSX file operations
//! including reading spreadsheets, extracting data, and writing Excel files.
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{DriverError, DriverResult, ensure_dir, file_exists, read_file_content, validate_path, write_file_content};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info};
/// Driver for reading Excel files
#[derive(Debug)]
pub struct ExcelReadDriver;
#[async_trait::async_trait]
impl Driver for ExcelReadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "excel_read";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Read data from Excel (.xlsx) files";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to read Excel spreadsheets, extract tabular data from .xlsx files";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the Excel file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("data.xlsx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "sheet".to_string(),
                param_type: "string".to_string(),
                description: "Sheet name or index (0-based)".to_string(),
                required: false,
                default: Some(Value::String("0".to_string())),
                example: Some(Value::String("Sheet1".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "has_header".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether the sheet has a header row".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
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
            "action": "excel_read",
            "parameters": {
                "path": "data.xlsx"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Sheet: Sheet1\nHeaders: [Name, Age]\nRow 1: [Alice, 30]\nRow 2: [Bob, 25]".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Document;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing excel_read driver");
        let start_time = Instant::now();
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name.clone());
            cb.on_log(task_id.clone(), driver_index, Some("Starting Excel read operation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(5), None);
        }
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("File path: {}", path)));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        let sheet_param = parameters.get("sheet").and_then(|v| v.as_str()).unwrap_or("0");
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Sheet: {}", sheet_param)));
            cb.on_progress(task_id.clone(), driver_index, Some(15), None);
        }
        let has_header = parameters.get("has_header").and_then(|v| v.as_bool()).unwrap_or(true);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Has header: {}", has_header)));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        let limit = parameters.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Row limit: {}", limit)));
            cb.on_progress(task_id.clone(), driver_index, Some(25), None);
        }
        debug!("Validating file path: {}", path);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Validating file path".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("Excel file not found: {}", path)));
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Opening Excel workbook".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        use calamine::{Reader, Xlsx, open_workbook};
        let mut workbook: Xlsx<_> =
            open_workbook(&validated_path).map_err(|e| DriverError::execution(format!("Failed to open Excel file: {}", e)))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Reading sheet names".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(45), None);
        }
        let sheet_names = workbook.sheet_names().to_vec();
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Available sheets: {:?}", sheet_names)));
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        let sheet_name = if let Ok(idx) = sheet_param.parse::<usize>() {
            if idx < sheet_names.len() {
                sheet_names[idx].clone()
            } else {
                return Err(DriverError::execution(format!("Sheet index {} out of range (max: {})", idx, sheet_names.len())));
            }
        } else {
            if sheet_names.contains(&sheet_param.to_string()) {
                sheet_param.to_string()
            } else {
                return Err(DriverError::execution(format!("Sheet '{}' not found. Available sheets: {:?}", sheet_param, sheet_names)));
            }
        };
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Reading sheet: {}", sheet_name)));
            cb.on_progress(task_id.clone(), driver_index, Some(55), None);
        }
        let mut sheet =
            workbook.worksheet_range(&sheet_name).map_err(|e| DriverError::execution(format!("Failed to read sheet '{}': {}", sheet_name, e)))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Sheet rows: {}", sheet.rows().len())));
            cb.on_progress(task_id.clone(), driver_index, Some(65), None);
        }
        let mut output = format!("Sheet: {}\n", sheet_name);
        if has_header && sheet.rows().len() > 0 {
            let header_row = sheet.rows().next().unwrap();
            let headers: Vec<String> = header_row.iter().map(|c| c.to_string()).collect();
            output.push_str(&format!("Headers: {:?}\n", headers));
            if let Some(cb) = callback {
                cb.on_log(task_id.clone(), driver_index, Some(format!("Headers: {:?}", headers)));
                cb.on_progress(task_id.clone(), driver_index, Some(70), None);
            }
        }
        let start_row = if has_header { 1 } else { 0 };
        let mut row_count = 0;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Reading rows".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(75), None);
        }
        for (i, row) in sheet.rows().skip(start_row).enumerate() {
            if i >= limit {
                output.push_str(&format!("... and {} more rows\n", sheet.rows().len() - i));
                break;
            }
            let row_data: Vec<String> = row.iter().map(|c| c.to_string()).collect();
            output.push_str(&format!("Row {}: {:?}\n", i + 1, row_data));
            row_count += 1;
            if i % 10 == 0 && i > 0 {
                if let Some(cb) = callback {
                    cb.on_progress(task_id.clone(), driver_index, Some(75 + (i * 20 / limit.min(100)) as u32), None);
                }
            }
        }
        output.push_str(&format!("Total rows read: {}\n", row_count));
        if let Some(cb) = callback {
            let duration = start_time.elapsed().as_millis() as u64;
            cb.on_log(task_id.clone(), driver_index, Some(format!("Read {} rows in {}ms", row_count, duration)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, step_name, Some(output.clone()));
        }
        info!("Excel read completed for: {} ({} rows)", path, row_count);
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Driver for writing Excel files
#[derive(Debug)]
pub struct ExcelWriteDriver;
#[async_trait::async_trait]
impl Driver for ExcelWriteDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "excel_write";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Write data to Excel (.xlsx) files";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to create Excel spreadsheets, export data to .xlsx format";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the Excel file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("output.xlsx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "headers".to_string(),
                param_type: "array".to_string(),
                description: "Column headers as an array of strings".to_string(),
                required: true,
                default: None,
                example: Some(json!(["Name", "Age", "City"])),
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
                name: "sheet_name".to_string(),
                param_type: "string".to_string(),
                description: "Name of the worksheet (default: 'Sheet1')".to_string(),
                required: false,
                default: Some(Value::String("Sheet1".to_string())),
                example: Some(Value::String("Data".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "excel_write",
            "parameters": {
                "path": "output.xlsx",
                "headers": ["Name", "Age"],
                "rows": [["Alice", "30"], ["Bob", "25"]]
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Excel written to: output.xlsx (2 rows)".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Document;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing excel_write driver");
        let start_time = Instant::now();
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name.clone());
            cb.on_log(task_id.clone(), driver_index, Some("Starting Excel write operation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(5), None);
        }
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("File path: {}", path)));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        let headers_json = parameters.get("headers").ok_or_else(|| DriverError::missing_parameter("headers"))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Parsing headers".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(15), None);
        }
        let headers = headers_json
            .as_array()
            .ok_or_else(|| DriverError::invalid_type("headers", "array", "other"))?
            .iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .collect::<Vec<_>>();
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Headers count: {}", headers.len())));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        let rows_json = parameters.get("rows").ok_or_else(|| DriverError::missing_parameter("rows"))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Parsing rows data".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(25), None);
        }
        let rows = rows_json.as_array().ok_or_else(|| DriverError::invalid_type("rows", "array", "other"))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Rows count: {}", rows.len())));
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        let sheet_name = parameters.get("sheet_name").and_then(|v| v.as_str()).unwrap_or("Sheet1");
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Sheet name: {}", sheet_name)));
            cb.on_progress(task_id.clone(), driver_index, Some(35), None);
        }
        debug!("Validating file path: {}", path);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Validating file path".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if let Some(parent) = validated_path.parent() {
            if let Some(cb) = callback {
                cb.on_log(task_id.clone(), driver_index, Some("Ensuring parent directory exists".to_string()));
                cb.on_progress(task_id.clone(), driver_index, Some(45), None);
            }
            ensure_dir(&parent.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to create directory: {}", e)))?;
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Creating Excel workbook".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        use rust_xlsxwriter::Workbook;
        let mut workbook = Workbook::new();
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Adding worksheet".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(55), None);
        }
        let worksheet = workbook.add_worksheet();
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Writing headers".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(60), None);
        }
        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string(0, col as u16, header).map_err(|e| DriverError::execution(format!("Failed to write header: {}", e)))?;
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Writing {} rows of data", rows.len())));
            cb.on_progress(task_id.clone(), driver_index, Some(65), None);
        }
        for (row, row_data) in rows.iter().enumerate() {
            let row_data_array = row_data.as_array().ok_or_else(|| DriverError::execution("Each row must be an array".to_string()))?;
            for (col, value) in row_data_array.iter().enumerate() {
                if let Some(s) = value.as_str() {
                    worksheet
                        .write_string((row + 1) as u32, col as u16, s)
                        .map_err(|e| DriverError::execution(format!("Failed to write string: {}", e)))?;
                } else if let Some(n) = value.as_f64() {
                    worksheet
                        .write_number((row + 1) as u32, col as u16, n)
                        .map_err(|e| DriverError::execution(format!("Failed to write number: {}", e)))?;
                } else if let Some(b) = value.as_bool() {
                    worksheet
                        .write_boolean((row + 1) as u32, col as u16, b)
                        .map_err(|e| DriverError::execution(format!("Failed to write boolean: {}", e)))?;
                } else {
                    worksheet
                        .write_string((row + 1) as u32, col as u16, &value.to_string())
                        .map_err(|e| DriverError::execution(format!("Failed to write value: {}", e)))?;
                }
            }
            if row % 10 == 0 && row > 0 {
                if let Some(cb) = callback {
                    cb.on_progress(task_id.clone(), driver_index, Some(65 + ((row * 25) / rows.len()) as u32), None);
                }
            }
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Saving Excel file".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(90), None);
        }
        workbook.save(&validated_path).map_err(|e| DriverError::execution(format!("Failed to save Excel file: {}", e)))?;
        let result = format!("Excel written to: {} ({} rows + header)", path, rows.len());
        if let Some(cb) = callback {
            let duration = start_time.elapsed().as_millis() as u64;
            cb.on_log(task_id.clone(), driver_index, Some(format!("Wrote {} rows in {}ms", rows.len(), duration)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, step_name, Some(result.clone()));
        }
        info!("Excel write completed: {} ({} rows)", path, rows.len());
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        parameters.get("headers").ok_or_else(|| DriverError::missing_parameter("headers"))?;
        parameters.get("rows").ok_or_else(|| DriverError::missing_parameter("rows"))?;
        return Ok(());
    }
}
