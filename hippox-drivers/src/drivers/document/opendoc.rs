use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{file_exists, validate_path};
use anyhow::Result;
use quick_xml::{Reader, events::Event};
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug)]
pub struct OdsReadDriver;

#[async_trait::async_trait]
impl Driver for OdsReadDriver {
    fn name(&self) -> &str {
        "ods_read"
    }

    fn description(&self) -> &str {
        "Read and extract data from OpenDocument Spreadsheet (.ods) files"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read OpenDocument spreadsheets (LibreOffice/OpenOffice Calc)"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the ODS file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("spreadsheet.ods".to_string())),
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
            "action": "ods_read",
            "parameters": {
                "path": "spreadsheet.ods"
            }
        })
    }

    fn example_output(&self) -> String {
        "Sheet: Sheet1\nRow 1: [Value1, Value2]\nRow 2: [Value3, Value4]".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Document
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let sheet_param = parameters
            .get("sheet")
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        let limit = parameters
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("ODS file not found: {}", path);
        }

        use std::fs::File;
        use zip::ZipArchive;
        let file = File::open(&validated_path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut content_xml = None;
        for i in 0..archive.len() {
            let entry = archive.by_index(i)?;
            if entry.name() == "content.xml" {
                let mut content = String::new();
                let mut reader = std::io::BufReader::new(entry);
                std::io::Read::read_to_string(&mut reader, &mut content)?;
                content_xml = Some(content);
                break;
            }
        }
        let content =
            content_xml.ok_or_else(|| anyhow::anyhow!("No content.xml found in ODS file"))?;
        let (sheets, sheet_names) = parse_ods_content(&content, limit)?;
        let sheet_data = if let Ok(idx) = sheet_param.parse::<usize>() {
            if idx < sheets.len() {
                &sheets[idx]
            } else {
                anyhow::bail!(
                    "Sheet index {} out of range (max: {})",
                    idx,
                    sheets.len() - 1
                )
            }
        } else {
            let sheet_name = sheet_param;
            if let Some(pos) = sheet_names.iter().position(|name| name == sheet_name) {
                &sheets[pos]
            } else {
                anyhow::bail!(
                    "Sheet '{}' not found. Available sheets: {:?}",
                    sheet_name,
                    sheet_names
                )
            }
        };
        let mut output = String::new();
        for (row_idx, row) in sheet_data.iter().enumerate() {
            output.push_str(&format!("Row {}: {:?}\n", row_idx + 1, row));
        }
        output.push_str(&format!("Total rows: {}", sheet_data.len()));
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
pub struct OdtReadDriver;

#[async_trait::async_trait]
impl Driver for OdtReadDriver {
    fn name(&self) -> &str {
        "odt_read"
    }

    fn description(&self) -> &str {
        "Read and extract text content from OpenDocument Text (.odt) files"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read OpenDocument text documents (LibreOffice/OpenOffice Writer)"
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the ODT file".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("document.odt".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "odt_read",
            "parameters": {
                "path": "document.odt"
            }
        })
    }

    fn example_output(&self) -> String {
        "Document content extracted from OpenDocument text file...".to_string()
    }

    fn category(&self) -> DriverCategory {
        DriverCategory::Document
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("ODT file not found: {}", path);
        }
        use std::fs::File;
        use zip::ZipArchive;
        let file = File::open(&validated_path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut content_xml = None;
        for i in 0..archive.len() {
            let entry = archive.by_index(i)?;
            if entry.name() == "content.xml" {
                let mut content = String::new();
                let mut reader = std::io::BufReader::new(entry);
                std::io::Read::read_to_string(&mut reader, &mut content)?;
                content_xml = Some(content);
                break;
            }
        }
        let content =
            content_xml.ok_or_else(|| anyhow::anyhow!("No content.xml found in ODT file"))?;
        let text = extract_text_from_odt_xml(&content);
        Ok(text)
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        Ok(())
    }
}

fn parse_ods_content(xml: &str, limit: usize) -> Result<(Vec<Vec<Vec<String>>>, Vec<String>)> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut sheets = Vec::new();
    let mut sheet_names = Vec::new();
    let mut current_sheet = Vec::new();
    let mut current_row = Vec::new();
    let mut in_table = false;
    let mut in_row = false;
    let mut in_cell = false;
    let mut cell_value = String::new();
    let mut current_sheet_name = String::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"table:table" => {
                    in_table = true;
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            if attr.key.as_ref() == b"table:name" {
                                if let Ok(name) = attr.unescape_value() {
                                    current_sheet_name = name.to_string();
                                }
                            }
                        }
                    }
                    current_sheet.clear();
                }
                b"table:table-row" => {
                    in_row = true;
                    current_row.clear();
                }
                b"table:table-cell" => {
                    in_cell = true;
                    cell_value.clear();
                }
                _ => {}
            },
            Ok(Event::Text(e)) => {
                if in_cell {
                    if let Ok(text) = e.decode() {
                        cell_value.push_str(&text);
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"table:table-cell" => {
                    in_cell = false;
                    current_row.push(cell_value.trim().to_string());
                }
                b"table:table-row" => {
                    in_row = false;
                    if !current_row.is_empty() && current_sheet.len() < limit {
                        current_sheet.push(current_row.clone());
                    }
                }
                b"table:table" => {
                    if !current_sheet.is_empty() {
                        sheets.push(current_sheet.clone());
                        sheet_names.push(current_sheet_name.clone());
                    }
                    in_table = false;
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("Error parsing XML: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }
    Ok((sheets, sheet_names))
}

fn extract_text_from_odt_xml(xml: &str) -> String {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut text_parts = Vec::new();
    let mut in_text = false;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"text:p" || e.name().as_ref() == b"text:h" {
                    text_parts.push("\n".to_string());
                }
            }
            Ok(Event::Text(e)) => {
                if let Ok(text) = e.decode() {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        text_parts.push(trimmed.to_string());
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("Error parsing XML: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }
    text_parts.join(" ")
}
