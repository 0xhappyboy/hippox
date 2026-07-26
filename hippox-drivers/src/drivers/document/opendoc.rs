//! OpenDocument file driver module
//!
//! This module provides drivers for OpenDocument file operations including
//! reading ODS (OpenDocument Spreadsheet) files and ODT (OpenDocument Text) files.
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{DriverError, DriverResult, file_exists, validate_path};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for reading ODS (OpenDocument Spreadsheet) files
#[derive(Debug)]
pub struct OdsReadDriver;
#[async_trait::async_trait]
impl Driver for OdsReadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "ods_read";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Read and extract data from OpenDocument Spreadsheet (.ods) files";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to read OpenDocument spreadsheets (LibreOffice/OpenOffice Calc)";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
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
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "ods_read",
            "parameters": {
                "path": "spreadsheet.ods"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Sheet: Sheet1\nRow 1: [Value1, Value2]\nRow 2: [Value3, Value4]".to_string();
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
        debug!("Executing ods_read driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let sheet_param = parameters.get("sheet").and_then(|v| v.as_str()).unwrap_or("0");
        let limit = parameters.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
        debug!("Reading ODS file: {}, sheet: {}, limit: {}", path, sheet_param, limit);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("ODS file not found: {}", path)));
        }
        use std::fs::File;
        use zip::ZipArchive;
        let file = File::open(&validated_path).map_err(|e| DriverError::execution(format!("Failed to open file: {}", e)))?;
        let mut archive = ZipArchive::new(file).map_err(|e| DriverError::execution(format!("Failed to open ODS archive: {}", e)))?;
        let mut content_xml = None;
        for i in 0..archive.len() {
            let entry = archive.by_index(i).map_err(|e| DriverError::execution(format!("Failed to read archive entry: {}", e)))?;
            if entry.name() == "content.xml" {
                let mut content = String::new();
                let mut reader = std::io::BufReader::new(entry);
                std::io::Read::read_to_string(&mut reader, &mut content)
                    .map_err(|e| DriverError::execution(format!("Failed to read content.xml: {}", e)))?;
                content_xml = Some(content);
                break;
            }
        }
        let content = content_xml.ok_or_else(|| DriverError::execution("No content.xml found in ODS file".to_string()))?;
        let (sheets, sheet_names) = parse_ods_content(&content, limit)?;
        let sheet_data = if let Ok(idx) = sheet_param.parse::<usize>() {
            if idx < sheets.len() {
                &sheets[idx]
            } else {
                return Err(DriverError::execution(format!("Sheet index {} out of range (max: {})", idx, sheets.len() - 1)));
            }
        } else {
            let sheet_name = sheet_param;
            if let Some(pos) = sheet_names.iter().position(|name| name == sheet_name) {
                &sheets[pos]
            } else {
                return Err(DriverError::execution(format!("Sheet '{}' not found. Available sheets: {:?}", sheet_name, sheet_names)));
            }
        };
        let mut output = String::new();
        for (row_idx, row) in sheet_data.iter().enumerate() {
            output.push_str(&format!("Row {}: {:?}\n", row_idx + 1, row));
        }
        output.push_str(&format!("Total rows: {}", sheet_data.len()));
        info!("ODS read completed: {} ({} rows)", path, sheet_data.len());
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Driver for reading ODT (OpenDocument Text) files
#[derive(Debug)]
pub struct OdtReadDriver;
#[async_trait::async_trait]
impl Driver for OdtReadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "odt_read";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Read and extract text content from OpenDocument Text (.odt) files";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to read OpenDocument text documents (LibreOffice/OpenOffice Writer)";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the ODT file".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("document.odt".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "odt_read",
            "parameters": {
                "path": "document.odt"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Document content extracted from OpenDocument text file...".to_string();
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
        debug!("Executing odt_read driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        debug!("Reading ODT file: {}", path);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("ODT file not found: {}", path)));
        }
        use std::fs::File;
        use zip::ZipArchive;
        let file = File::open(&validated_path).map_err(|e| DriverError::execution(format!("Failed to open file: {}", e)))?;
        let mut archive = ZipArchive::new(file).map_err(|e| DriverError::execution(format!("Failed to open ODT archive: {}", e)))?;
        let mut content_xml = None;
        for i in 0..archive.len() {
            let entry = archive.by_index(i).map_err(|e| DriverError::execution(format!("Failed to read archive entry: {}", e)))?;
            if entry.name() == "content.xml" {
                let mut content = String::new();
                let mut reader = std::io::BufReader::new(entry);
                std::io::Read::read_to_string(&mut reader, &mut content)
                    .map_err(|e| DriverError::execution(format!("Failed to read content.xml: {}", e)))?;
                content_xml = Some(content);
                break;
            }
        }
        let content = content_xml.ok_or_else(|| DriverError::execution("No content.xml found in ODT file".to_string()))?;
        let text = extract_text_from_odt_xml(&content);
        info!("ODT read completed: {} ({} characters)", path, text.len());
        return Ok(text);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Parses ODS content XML and extracts sheet data
///
/// # Arguments
/// * `xml` - ODS content XML
/// * `limit` - Maximum number of rows to read per sheet
///
/// # Returns
/// * `DriverResult<(Vec<Vec<Vec<String>>>, Vec<String>)>` - Sheets data and sheet names
fn parse_ods_content(xml: &str, limit: usize) -> DriverResult<(Vec<Vec<Vec<String>>>, Vec<String>)> {
    use quick_xml::{Reader, events::Event};
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
                debug!("Error parsing XML: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }
    return Ok((sheets, sheet_names));
}
/// Extracts text content from ODT XML
///
/// # Arguments
/// * `xml` - ODT content XML
///
/// # Returns
/// * `String` - Extracted text content
fn extract_text_from_odt_xml(xml: &str) -> String {
    use quick_xml::{Reader, events::Event};
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut text_parts = Vec::new();
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
                debug!("Error parsing XML: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }
    return text_parts.join(" ");
}
