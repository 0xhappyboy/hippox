//! PDF file driver module
//!
//! This module provides drivers for PDF file operations including
//! reading PDF files, extracting text content, merging PDF files,
//! and getting PDF metadata information.
use crate::DriverCallback;
use crate::DriverContext;
use crate::{
    DriverCategory,
    types::{Driver, DriverParameter},
};
use crate::{DriverError, DriverResult, ensure_dir, file_exists, validate_path};
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Driver for reading PDF files and extracting text content
#[derive(Debug)]
pub struct PdfReadDriver;
#[async_trait::async_trait]
impl Driver for PdfReadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "pdf_read";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Read and extract text content from PDF files";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to read PDF documents, extract text from PDF files";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the PDF file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("document.pdf".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "start_page".to_string(),
                param_type: "integer".to_string(),
                description: "Starting page number (1-indexed)".to_string(),
                required: false,
                default: Some(Value::Number(1.into())),
                example: Some(Value::Number(1.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "end_page".to_string(),
                param_type: "integer".to_string(),
                description: "Ending page number (inclusive)".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "pdf_read",
            "parameters": {
                "path": "document.pdf"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "PDF content extracted from document.pdf\nPage 1: This is the content...".to_string();
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
        debug!("Executing pdf_read driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name.clone());
            cb.on_log(task_id.clone(), driver_index, Some("Starting PDF read operation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Reading PDF file: {}", path)));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        let start_page = parameters.get("start_page").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        let end_page = parameters.get("end_page").and_then(|v| v.as_u64()).map(|v| v as usize);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Start page: {}, End page: {:?}", start_page, end_page)));
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("PDF file not found: {}", path)));
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Validated path, loading PDF".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(40), None);
        }
        use pdf_extract::extract_text;
        let full_text = extract_text(&validated_path).map_err(|e| DriverError::execution(format!("Failed to extract PDF text: {}", e)))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("PDF text extracted successfully".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(60), None);
        }
        let pages: Vec<&str> = full_text.split("\n\n").collect();
        let start = start_page.saturating_sub(1);
        let end = end_page.unwrap_or(pages.len()).min(pages.len());
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Total pages: {}, Selected range: {}-{}", pages.len(), start + 1, end)));
            cb.on_progress(task_id.clone(), driver_index, Some(75), None);
        }
        if start >= pages.len() {
            return Err(DriverError::execution(format!("Start page {} exceeds total pages {}", start_page, pages.len())));
        }
        let mut output = format!("PDF file: {}\n", path);
        output.push_str(&format!("Total pages: {}\n", pages.len()));
        output.push_str(&format!("Showing pages {}-{}\n\n", start + 1, end));
        for i in start..end {
            output.push_str(&format!("=== Page {} ===\n", i + 1));
            output.push_str(pages[i]);
            output.push_str("\n\n");
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Completed PDF read, output length: {} bytes", output.len())));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("pdf_read".to_string()), Some(output.clone()));
        }
        info!("PDF read completed: {}", path);
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Driver for merging multiple PDF files into one
#[derive(Debug)]
pub struct PdfMergeDriver;
#[async_trait::async_trait]
impl Driver for PdfMergeDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "pdf_merge";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Merge multiple PDF files into a single PDF";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to combine multiple PDF files into one";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "inputs".to_string(),
                param_type: "array".to_string(),
                description: "Array of PDF file paths to merge".to_string(),
                required: true,
                default: None,
                example: Some(json!(["file1.pdf", "file2.pdf", "file3.pdf"])),
                enum_values: None,
            },
            DriverParameter {
                name: "output".to_string(),
                param_type: "string".to_string(),
                description: "Output PDF file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("merged.pdf".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "pdf_merge",
            "parameters": {
                "inputs": ["doc1.pdf", "doc2.pdf"],
                "output": "merged.pdf"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Merged 2 PDF files into: merged.pdf".to_string();
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
        debug!("Executing pdf_merge driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name.clone());
            cb.on_log(task_id.clone(), driver_index, Some("Starting PDF merge operation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        use lopdf::{Document, Object, ObjectId};
        let inputs = parameters
            .get("inputs")
            .ok_or_else(|| DriverError::missing_parameter("inputs"))?
            .as_array()
            .ok_or_else(|| DriverError::invalid_type("inputs", "array", "other"))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Number of input files: {}", inputs.len())));
            cb.on_progress(task_id.clone(), driver_index, Some(15), None);
        }
        let output = parameters.get("output").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("output"))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Output file: {}", output)));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        if inputs.is_empty() {
            return Err(DriverError::execution("At least one input file is required".to_string()));
        }
        let validated_output = validate_path(output, None).map_err(|e| DriverError::execution(format!("Invalid output path: {}", e)))?;
        if let Some(parent) = validated_output.parent() {
            ensure_dir(&parent.to_string_lossy()).map_err(|e| DriverError::execution(format!("Failed to create directory: {}", e)))?;
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Validated output path, creating directory".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(25), None);
        }
        let mut merged_doc = Document::new();
        let mut total_pages = 0;
        let mut max_id = 0;
        let total_inputs = inputs.len();
        for (idx, input_path) in inputs.iter().enumerate() {
            let path = input_path.as_str().ok_or_else(|| DriverError::execution("Input path must be a string".to_string()))?;
            if let Some(cb) = callback {
                let progress = 25 + ((idx + 1) as f32 / total_inputs as f32 * 50.0) as u32;
                cb.on_log(task_id.clone(), driver_index, Some(format!("Processing file {}/{}: {}", idx + 1, total_inputs, path)));
                cb.on_progress(task_id.clone(), driver_index, Some(progress), None);
            }
            let validated_input = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid input path '{}': {}", path, e)))?;
            let doc = Document::load(&validated_input).map_err(|e| DriverError::execution(format!("Failed to load PDF '{}': {}", path, e)))?;
            let pages = doc.page_iter().collect::<Vec<_>>();
            total_pages += pages.len();
            if let Some(cb) = callback {
                cb.on_log(task_id.clone(), driver_index, Some(format!("Loaded PDF with {} pages", pages.len())));
            }
            for (id, object) in doc.objects.iter() {
                let new_id = (id.0 + max_id, id.1 + max_id as u16);
                merged_doc.objects.insert(new_id, object.clone());
            }
            max_id += doc.max_id;
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("All files loaded, total pages: {}", total_pages)));
            cb.on_progress(task_id.clone(), driver_index, Some(80), None);
        }
        let mut page_objects = Vec::new();
        for (object_id, object) in merged_doc.objects.iter() {
            if let Ok(dict) = object.as_dict() {
                if let Ok(value) = dict.get(b"Type") {
                    if let Ok(name) = value.as_name() {
                        if name == b"Page" {
                            page_objects.push(*object_id);
                        }
                    }
                }
            }
        }
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Found {} page objects", page_objects.len())));
            cb.on_progress(task_id.clone(), driver_index, Some(90), None);
        }
        if page_objects.is_empty() {
            return Err(DriverError::execution("No pages found in input PDFs".to_string()));
        }
        merged_doc.save(&validated_output).map_err(|e| DriverError::execution(format!("Failed to save merged PDF: {}", e)))?;
        let result = format!("Merged {} PDF files into: {} ({} total pages)", inputs.len(), output, total_pages);
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Merge completed: {}", result)));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("pdf_merge".to_string()), Some(result.clone()));
        }
        info!("PDF merge completed: {} files merged into {}", inputs.len(), output);
        return Ok(result);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("inputs").ok_or_else(|| DriverError::missing_parameter("inputs"))?;
        parameters.get("output").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("output"))?;
        return Ok(());
    }
}
/// Driver for getting PDF metadata information
#[derive(Debug)]
pub struct PdfInfoDriver;
#[async_trait::async_trait]
impl Driver for PdfInfoDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "pdf_info";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get metadata information from PDF file (pages, title, author, etc.)";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to get information about a PDF file";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the PDF file".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("document.pdf".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "pdf_info",
            "parameters": {
                "path": "document.pdf"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "PDF Info:\nPages: 25\nTitle: My Document\nAuthor: John Doe".to_string();
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
        debug!("Executing pdf_info driver");
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let driver_index = context.as_ref().and_then(|c| c.driver_index());
        let step_name = context.as_ref().and_then(|c| c.driver_name()).map(String::from);
        // Notify callback of start
        if let Some(cb) = callback {
            cb.on_start(task_id.clone(), driver_index, step_name.clone());
            cb.on_log(task_id.clone(), driver_index, Some("Starting PDF info operation".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(10), None);
        }
        use lopdf::Document;
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Reading PDF info for: {}", path)));
            cb.on_progress(task_id.clone(), driver_index, Some(20), None);
        }
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("Validated path, loading PDF".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(30), None);
        }
        let doc = Document::load(&validated_path).map_err(|e| DriverError::execution(format!("Failed to load PDF: {}", e)))?;
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some("PDF loaded successfully".to_string()));
            cb.on_progress(task_id.clone(), driver_index, Some(50), None);
        }
        let pages = doc.page_iter().count();
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("Total pages: {}", pages)));
            cb.on_progress(task_id.clone(), driver_index, Some(60), None);
        }
        let mut output = format!("PDF Info for: {}\n", path);
        output.push_str(&format!("Total pages: {}\n", pages));
        if let Ok(info_ref) = doc.trailer.get(b"Info") {
            if let Ok(info_id) = info_ref.as_reference() {
                if let Ok(info) = doc.get_object(info_id) {
                    if let Ok(dict) = info.as_dict() {
                        if let Some(cb) = callback {
                            cb.on_log(task_id.clone(), driver_index, Some("Extracting metadata from PDF".to_string()));
                            cb.on_progress(task_id.clone(), driver_index, Some(70), None);
                        }
                        if let Ok(title) = dict.get(b"Title") {
                            if let Ok(title_str) = title.as_str() {
                                output.push_str(&format!("Title: {:?}\n", title_str));
                            }
                        }
                        if let Ok(author) = dict.get(b"Author") {
                            if let Ok(author_str) = author.as_str() {
                                output.push_str(&format!("Author: {:?}\n", author_str));
                            }
                        }
                        if let Ok(subject) = dict.get(b"Subject") {
                            if let Ok(subject_str) = subject.as_str() {
                                output.push_str(&format!("Subject: {:?}\n", subject_str));
                            }
                        }
                        if let Ok(creator) = dict.get(b"Creator") {
                            if let Ok(creator_str) = creator.as_str() {
                                output.push_str(&format!("Creator: {:?}\n", creator_str));
                            }
                        }
                        if let Ok(producer) = dict.get(b"Producer") {
                            if let Ok(producer_str) = producer.as_str() {
                                output.push_str(&format!("Producer: {:?}\n", producer_str));
                            }
                        }
                    }
                }
            }
        }
        let file_size = std::fs::metadata(&validated_path).map(|m| m.len()).unwrap_or(0);
        output.push_str(&format!("File size: {:.2} KB\n", file_size as f64 / 1024.0));
        if let Some(cb) = callback {
            cb.on_log(task_id.clone(), driver_index, Some(format!("PDF info completed, output length: {} bytes", output.len())));
            cb.on_progress(task_id.clone(), driver_index, Some(100), None);
            cb.on_complete(task_id.clone(), driver_index, Some("pdf_info".to_string()), Some(output.clone()));
        }
        info!("PDF info completed: {}", path);
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
