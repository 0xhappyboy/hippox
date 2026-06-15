use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::{
    ensure_dir, file_exists, read_file_content,
    types::{Skill, SkillParameter},
    validate_path, write_file_content,
};

#[derive(Debug)]
pub struct CsvReadSkill;

#[async_trait::async_trait]
impl Skill for CsvReadSkill {
    fn name(&self) -> &str {
        "csv_read"
    }

    fn description(&self) -> &str {
        "Read and parse CSV file content into structured data"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read a CSV file, analyze tabular data, or extract information from spreadsheets"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the CSV file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("data.csv".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "has_header".to_string(),
                param_type: "boolean".to_string(),
                description: "Whether the CSV has a header row".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
            SkillParameter {
                name: "delimiter".to_string(),
                param_type: "string".to_string(),
                description: "CSV delimiter character (default: ',')".to_string(),
                required: false,
                default: Some(Value::String(",".to_string())),
                example: Some(Value::String(";".to_string())),
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
            "action": "csv_read",
            "parameters": {
                "path": "data.csv"
            }
        })
    }

    fn example_output(&self) -> String {
        "Header: [name, age, city]\nRow 1: [Alice, 30, Beijing]\nRow 2: [Bob, 25, Shanghai]"
            .to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let has_header = parameters
            .get("has_header")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let delimiter = parameters
            .get("delimiter")
            .and_then(|v| v.as_str())
            .unwrap_or(",")
            .chars()
            .next()
            .unwrap_or(',');
        let limit = parameters
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("CSV file not found: {}", path);
        }
        let content = read_file_content(&validated_path.to_string_lossy())?;
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(has_header)
            .delimiter(delimiter as u8)
            .from_reader(content.as_bytes());
        let headers: Vec<String> = if has_header {
            reader.headers()?.iter().map(|h| h.to_string()).collect()
        } else {
            (0..reader.headers()?.len())
                .map(|i| format!("Column_{}", i + 1))
                .collect()
        };
        let mut rows = Vec::new();
        for (i, result) in reader.records().enumerate() {
            if i >= limit {
                rows.push(format!(
                    "... and {} more rows",
                    reader.records().count() - i
                ));
                break;
            }
            let record = result?;
            let row: Vec<String> = record.iter().map(|f| f.to_string()).collect();
            rows.push(format!("{:?}", row));
        }
        let mut output = format!("Headers: {:?}\n", headers);
        for (i, row) in rows.iter().enumerate() {
            output.push_str(&format!("Row {}: {}\n", i + 1, row));
        }
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
pub struct CsvWriteSkill;

#[async_trait::async_trait]
impl Skill for CsvWriteSkill {
    fn name(&self) -> &str {
        "csv_write"
    }

    fn description(&self) -> &str {
        "Write structured data to a CSV file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to save tabular data, export to CSV, or create a spreadsheet"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to save the CSV file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("output.csv".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "headers".to_string(),
                param_type: "array".to_string(),
                description: "Column headers as an array of strings".to_string(),
                required: true,
                default: None,
                example: Some(json!(["name", "age", "city"])),
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
                name: "delimiter".to_string(),
                param_type: "string".to_string(),
                description: "CSV delimiter character (default: ',')".to_string(),
                required: false,
                default: Some(Value::String(",".to_string())),
                example: Some(Value::String(";".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "csv_write",
            "parameters": {
                "path": "output.csv",
                "headers": ["name", "age"],
                "rows": [["Alice", "30"], ["Bob", "25"]]
            }
        })
    }

    fn example_output(&self) -> String {
        "CSV written to: output.csv (2 rows)".to_string()
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
        let delimiter = parameters
            .get("delimiter")
            .and_then(|v| v.as_str())
            .unwrap_or(",")
            .chars()
            .next()
            .unwrap_or(',');
        let validated_path = validate_path(path, None)?;
        if let Some(parent) = validated_path.parent() {
            ensure_dir(&parent.to_string_lossy())?;
        }
        let mut csv_content = String::new();
        csv_content.push_str(&headers.join(&delimiter.to_string()));
        csv_content.push('\n');
        for row in rows {
            let row_array = row
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Each row must be an array"))?;
            let row_str: Vec<String> = row_array
                .iter()
                .map(|v| v.as_str().unwrap_or("").to_string())
                .collect();
            csv_content.push_str(&row_str.join(&delimiter.to_string()));
            csv_content.push('\n');
        }
        write_file_content(&validated_path.to_string_lossy(), &csv_content, false)?;
        Ok(format!("CSV written to: {} ({} rows)", path, rows.len()))
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
