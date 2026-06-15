use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::{ensure_dir, file_exists, types::{Skill, SkillParameter}, validate_path};

#[derive(Debug)]
pub struct ExcelReadSkill;

#[async_trait::async_trait]
impl Skill for ExcelReadSkill {
    fn name(&self) -> &str {
        "excel_read"
    }

    fn description(&self) -> &str {
        "Read data from Excel (.xlsx) files"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read Excel spreadsheets, extract tabular data from .xlsx files"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the Excel file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("data.xlsx".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "sheet".to_string(),
                param_type: "string".to_string(),
                description: "Sheet name or index (0-based)".to_string(),
                required: false,
                default: Some(Value::String("0".to_string())),
                example: Some(Value::String("Sheet1".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "has_header".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether the sheet has a header row".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of rows to read".to_string(),
                required: false,
                default: Some(Value::Number(100.into())),
                example: Some(Value::Number(50.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "excel_read",
            "parameters": {
                "path": "data.xlsx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Sheet: Sheet1\nHeaders: [Name, Age]\nRow 1: [Alice, 30]\nRow 2: [Bob, 25]".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let sheet_param = parameters
            .get("sheet")
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        let has_header = parameters
            .get("has_header")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let limit = parameters
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("Excel file not found: {}", path);
        }
        // Use calamine to read Excel
        use calamine::{Reader, Xlsx, open_workbook};
        let mut workbook: Xlsx<_> = open_workbook(&validated_path)?;
        // Get sheet by name or index
        let sheet_names = workbook.sheet_names().to_vec();
        let sheet_name = if let Ok(idx) = sheet_param.parse::<usize>() {
            if idx < sheet_names.len() {
                sheet_names[idx].clone()
            } else {
                anyhow::bail!(
                    "Sheet index {} out of range (max: {})",
                    idx,
                    sheet_names.len()
                )
            }
        } else {
            if sheet_names.contains(&sheet_param.to_string()) {
                sheet_param.to_string()
            } else {
                anyhow::bail!(
                    "Sheet '{}' not found. Available sheets: {:?}",
                    sheet_param,
                    sheet_names
                )
            }
        };
        let mut sheet = workbook
            .worksheet_range(&sheet_name)
            .map_err(|e| anyhow::anyhow!("Failed to read sheet '{}': {}", sheet_name, e))?;
        let mut output = format!("Sheet: {}\n", sheet_name);
        if has_header && sheet.rows().len() == 0 {
            let header_row = sheet.rows().next().unwrap();
            let headers: Vec<String> = header_row.iter().map(|c| c.to_string()).collect();
            output.push_str(&format!("Headers: {:?}\n", headers));
        }
        let start_row = if has_header { 1 } else { 0 };
        let mut row_count = 0;
        for (i, row) in sheet.rows().skip(start_row).enumerate() {
            if i >= limit {
                output.push_str(&format!("... and {} more rows\n", sheet.rows().len() - i));
                break;
            }
            let row_data: Vec<String> = row.iter().map(|c| c.to_string()).collect();
            output.push_str(&format!("Row {}: {:?}\n", i + 1, row_data));
            row_count += 1;
        }
        output.push_str(&format!("Total rows read: {}\n", row_count));
        Ok(output)
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ExcelWriteSkill;

#[async_trait::async_trait]
impl Skill for ExcelWriteSkill {
    fn name(&self) -> &str {
        "excel_write"
    }

    fn description(&self) -> &str {
        "Write data to Excel (.xlsx) files"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to create Excel spreadsheets, export data to .xlsx format"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the Excel file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("output.xlsx".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "headers".to_string(),
                param_type: "array".to_string(),
                description: "Column headers as an array of strings".to_string(),
                required: true,
                default: None,
                example: Some(json!(["Name", "Age", "City"])),
                enum_values: None,
            },
            SkillParameter {
                name: "rows".to_string(),
                param_type: "array".to_string(),
                description: "Data rows as array of arrays".to_string(),
                required: true,
                default: None,
                example: Some(json!([
                    ["Alice", "30", "Beijing"],
                    ["Bob", "25", "Shanghai"]
                ])),
                enum_values: None,
            },
            SkillParameter {
                name: "sheet_name".to_string(),
                param_type: "string".to_string(),
                description: "Name of the worksheet (default: 'Sheet1')".to_string(),
                required: false,
                default: Some(Value::String("Sheet1".to_string())),
                example: Some(Value::String("Data".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "excel_write",
            "parameters": {
                "path": "output.xlsx",
                "headers": ["Name", "Age"],
                "rows": [["Alice", "30"], ["Bob", "25"]]
            }
        })
    }

    fn example_output(&self) -> String {
        "Excel written to: output.xlsx (2 rows)".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let headers_json = parameters
            .get("headers")
            .ok_or_else(|| anyhow::anyhow!("Missing 'headers' parameter"))?;
        let rows_json = parameters
            .get("rows")
            .ok_or_else(|| anyhow::anyhow!("Missing 'rows' parameter"))?;
        let headers = headers_json
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("'headers' must be an array"))?
            .iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .collect::<Vec<_>>();
        let rows = rows_json
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("'rows' must be an array"))?;
        let sheet_name = parameters
            .get("sheet_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Sheet1");
        let validated_path = validate_path(path, None)?;
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy())?;
        }
        use rust_xlsxwriter::{Format, Workbook};
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string(0, col as u16, header)?;
        }
        for (row, row_data) in rows.iter().enumerate() {
            let row_data_array = row_data
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Each row must be an array"))?;
            for (col, value) in row_data_array.iter().enumerate() {
                if let Some(s) = value.as_str() {
                    worksheet.write_string((row + 1) as u32, col as u16, s)?;
                } else if let Some(n) = value.as_f64() {
                    worksheet.write_number((row + 1) as u32, col as u16, n)?;
                } else if let Some(b) = value.as_bool() {
                    worksheet.write_boolean((row + 1) as u32, col as u16, b)?;
                } else {
                    worksheet.write_string((row + 1) as u32, col as u16, &value.to_string())?;
                }
            }
        }
        workbook.save(&validated_path)?;
        Ok(format!(
            "Excel written to: {} ({} rows + header)",
            path,
            rows.len()
        ))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        parameters
            .get("headers")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: headers"))?;
        parameters
            .get("rows")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: rows"))?;
        Ok(())
    }
}
