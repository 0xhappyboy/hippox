//! Word document (DOCX) driver module
//!
//! This module provides drivers for Microsoft Word DOCX file operations
//! including reading text content, extracting metadata, and parsing
//! document structure.
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{DriverError, DriverResult, ensure_dir, file_exists, read_file_content, validate_path, write_file_content};
/// Driver for reading DOCX files and extracting text content
#[derive(Debug)]
pub struct DocxReadDriver;
#[async_trait::async_trait]
impl Driver for DocxReadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "docx_read";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Read and extract text content from Word (.docx) files";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to read Microsoft Word documents, extract text, or convert DOCX to plain text";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the DOCX file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("document.docx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "include_tables".to_string(),
                param_type: "boolean".to_string(),
                description: "Include table data in output".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "docx_read",
            "parameters": {
                "path": "document.docx"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Document content extracted from Word file...".to_string();
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
        debug!("Executing docx_read driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name.clone());
            cb.on_log(task_id.clone(), driver_index, Some("Starting DOCX read operation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Reading DOCX file: {}", path)));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        let include_tables = parameters.get("include_tables").and_then(|v| v.as_bool()).unwrap_or(true);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Include tables: {}", include_tables)));
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        debug!("Validating file path: {}", path);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("DOCX file not found: {}", path)));
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Validated path, opening DOCX file".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        debug!("Opening DOCX archive: {}", path);
        use std::fs::File;
        use zip::ZipArchive;
        let file = File::open(&validated_path).map_err(|e| DriverError::execution(format!("Failed to open file: {}", e)))?;
        let mut archive = ZipArchive::new(file).map_err(|e| DriverError::execution(format!("Failed to open DOCX archive: {}", e)))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("DOCX archive opened, entries: {}", archive.len())));
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        let mut document_content = None;
        for i in 0..archive.len() {
            let entry = archive.by_index(i).map_err(|e| DriverError::execution(format!("Failed to read archive entry: {}", e)))?;
            if entry.name() == "word/document.xml" {
                if let Some(cb) = callback {
                    cb.on_log(task_id.clone(), driver_index, Some("Found word/document.xml".to_string()));
                    cb.on_progress(task_id.clone(), driver_index, Some(60), None);
                }
                let mut content = String::new();
                let mut reader = std::io::BufReader::new(entry);
                std::io::Read::read_to_string(&mut reader, &mut content)
                    .map_err(|e| DriverError::execution(format!("Failed to read document XML: {}", e)))?;
                document_content = Some(content);
                break;
            }
        }
        let content = document_content.ok_or_else(|| DriverError::execution("No document.xml found in DOCX file".to_string()))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Document XML loaded, size: {} bytes", content.len())));
            cb.on_progress(task_id.clone(), driver_index, Some(75), None);
        }
        debug!("Extracting text from DOCX XML");
        let text = extract_text_from_docx_xml(&content, include_tables);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Text extracted, length: {} characters", text.len())));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("docx_read".to_string()), Some(text.clone()));
        }
        info!("DOCX read completed successfully: {}", path);
        return Ok(text);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Driver for getting DOCX file metadata and information
#[derive(Debug)]
pub struct DocxInfoDriver;
#[async_trait::async_trait]
impl Driver for DocxInfoDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "docx_info";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get metadata and structure information about a Word document";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to get document properties, word count, or file info";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the DOCX file".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("document.docx".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "docx_info",
            "parameters": {
                "path": "document.docx"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Word count: 1500\nPages: 5\nFile size: 120 KB".to_string();
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
        debug!("Executing docx_info driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name);
            cb.on_log(task_id.clone(), driver_index, Some("Starting DOCX info operation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Getting DOCX info for: {}", path)));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        debug!("Validating file path: {}", path);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("DOCX file not found: {}", path)));
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Validated path, reading file metadata".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        debug!("Opening DOCX archive: {}", path);
        use std::fs::File;
        use zip::ZipArchive;
        let file = File::open(&validated_path).map_err(|e| DriverError::execution(format!("Failed to open file: {}", e)))?;
        let mut archive = ZipArchive::new(file).map_err(|e| DriverError::execution(format!("Failed to open DOCX archive: {}", e)))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("DOCX archive opened, entries: {}", archive.len())));
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        let metadata = std::fs::metadata(&validated_path).map_err(|e| DriverError::execution(format!("Failed to read file metadata: {}", e)))?;
        let file_size = metadata.len();
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("File size: {:.2} KB", file_size as f64 / 1024.0)));
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        let mut document_content = None;
        for i in 0..archive.len() {
            let entry = archive.by_index(i).map_err(|e| DriverError::execution(format!("Failed to read archive entry: {}", e)))?;
            if entry.name() == "word/document.xml" {
                if let Some(cb) = callback {
                    cb.on_log(task_id.clone(), driver_index, Some("Found word/document.xml".to_string()));
                    cb.on_progress(task_id.clone(), driver_index, Some(60), None);
                }
                let mut content = String::new();
                let mut reader = std::io::BufReader::new(entry);
                std::io::Read::read_to_string(&mut reader, &mut content)
                    .map_err(|e| DriverError::execution(format!("Failed to read document XML: {}", e)))?;
                document_content = Some(content);
                break;
            }
        }
        let mut output = String::new();
        output.push_str(&format!("File: {}\n", path));
        output.push_str(&format!("File size: {:.2} KB\n", file_size as f64 / 1024.0));
        if let Some(content) = document_content {
            if let Some(cb) = callback {
                cb.on_log(task_id.clone(), driver_index, Some("Extracting text from document XML".to_string()));
                cb.on_progress(task_id.clone(), driver_index, Some(70), None);
            }
            let text = extract_text_from_docx_xml(&content, false);
            let word_count = text.split_whitespace().count();
            let char_count = text.chars().count();
            let line_count = text.lines().count();
            if let Some(cb) = callback {
                cb.on_log(
                    task_id.clone(),
                    driver_index,
                    Some(format!("Word count: {}, Character count: {}, Line count: {}", word_count, char_count, line_count)),
                );
                cb.on_progress(task_id.clone(), driver_index, Some(80), None);
            }
            output.push_str(&format!("Word count: {}\n", word_count));
            output.push_str(&format!("Character count: {}\n", char_count));
            output.push_str(&format!("Line count: {}\n", line_count));
        } else {
            output.push_str("Unable to extract document content\n");
            if let Some(cb) = callback {
                cb.on_log(task_id.clone(), driver_index, Some("Unable to extract document content".to_string()));
            }
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("DOCX info completed".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("docx_info".to_string()), Some(output.clone()));
        }
        info!("DOCX info completed for: {}", path);
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Extracts text content from DOCX XML
///
/// # Arguments
/// * `xml` - DOCX document XML content
/// * `include_tables` - Whether to include table data in the output
///
/// # Returns
/// * `String` - Extracted text content
fn extract_text_from_docx_xml(xml: &str, include_tables: bool) -> String {
    use quick_xml::{Reader, events::Event};
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut text_parts = Vec::new();
    let mut in_text = false;
    let mut in_table = false;
    let mut table_content = Vec::new();
    let mut current_row = Vec::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"w:t" => in_text = true,
                b"w:tbl" => {
                    if include_tables {
                        in_table = true;
                    }
                }
                b"w:tr" if in_table => {
                    current_row.clear();
                }
                _ => {}
            },
            Ok(Event::Text(e)) => {
                if in_text {
                    if let Ok(text) = e.decode() {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() && !in_table {
                            text_parts.push(trimmed.to_string());
                        } else if in_table && include_tables {
                            current_row.push(trimmed.to_string());
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"w:t" => in_text = false,
                b"w:tr" if in_table => {
                    if include_tables && !current_row.is_empty() {
                        table_content.push(current_row.clone());
                    }
                    current_row.clear();
                }
                b"w:tbl" => {
                    if include_tables && !table_content.is_empty() {
                        text_parts.push(format_table(&table_content));
                        table_content.clear();
                    }
                    in_table = false;
                }
                b"w:p" => {
                    if !in_table {
                        text_parts.push("\n".to_string());
                    }
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
    return text_parts.join(" ");
}
/// Formats a table for text output
///
/// # Arguments
/// * `table` - Table data as a vector of rows
///
/// # Returns
/// * `String` - Formatted table string
fn format_table(table: &[Vec<String>]) -> String {
    if table.is_empty() {
        return String::new();
    }
    let mut output = String::from("\n[TABLE]\n");
    for row in table {
        output.push_str("| ");
        for cell in row {
            output.push_str(&format!("{} | ", cell));
        }
        output.push('\n');
    }
    output.push_str("[/TABLE]\n");
    return output;
}
