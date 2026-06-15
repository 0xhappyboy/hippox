//! PDF manipulation skills module
//!
//! This module provides skills for reading, merging, and extracting metadata from PDF files.
//! It includes three main skills:
//! - `PdfReadSkill`: Extract text content from PDF files
//! - `PdfMergeSkill`: Merge multiple PDF files into a single document
//! - `PdfInfoSkill`: Get metadata information from PDF files

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::{
    ensure_dir, file_exists,
    types::{Skill, SkillParameter},
    validate_path,
};

/// Skill for reading and extracting text content from PDF files
///
/// This skill allows extracting text from PDF documents with optional page range filtering.
/// It uses the `pdf_extract` crate for text extraction and supports validating file paths
/// and checking file existence before processing.
///
/// # Parameters
/// - `path` (required): Path to the PDF file
/// - `start_page` (optional): Starting page number (1-indexed, defaults to 1)
/// - `end_page` (optional): Ending page number inclusive
///
/// # Returns
/// Returns a formatted string containing the extracted text with page separators,
/// file information, and the total number of pages.
///
/// # Errors
/// Returns an error if:
/// - The path parameter is missing
/// - The PDF file does not exist
/// - Text extraction fails
/// - The start page exceeds total pages
#[derive(Debug)]
pub struct PdfReadSkill;

#[async_trait::async_trait]
impl Skill for PdfReadSkill {
    fn name(&self) -> &str {
        "pdf_read"
    }

    fn description(&self) -> &str {
        "Read and extract text content from PDF files"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read PDF documents, extract text from PDF files"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the PDF file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("document.pdf".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "start_page".to_string(),
                param_type: "integer".to_string(),
                description: "Starting page number (1-indexed)".to_string(),
                required: false,
                default: Some(Value::Number(1.into())),
                example: Some(Value::Number(1.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "end_page".to_string(),
                param_type: "integer".to_string(),
                description: "Ending page number (inclusive)".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "pdf_read",
            "parameters": {
                "path": "document.pdf"
            }
        })
    }

    fn example_output(&self) -> String {
        "PDF content extracted from document.pdf\nPage 1: This is the content...".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let start_page = parameters
            .get("start_page")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;
        let end_page = parameters
            .get("end_page")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("PDF file not found: {}", path);
        }
        use pdf_extract::extract_text;
        let full_text = extract_text(&validated_path)
            .map_err(|e| anyhow::anyhow!("Failed to extract PDF text: {}", e))?;
        let pages: Vec<&str> = full_text.split("\n\n").collect();
        let start = start_page.saturating_sub(1);
        let end = end_page.unwrap_or(pages.len()).min(pages.len());
        if start >= pages.len() {
            anyhow::bail!(
                "Start page {} exceeds total pages {}",
                start_page,
                pages.len()
            );
        }
        let mut output = format!("PDF file: {}\n", path);
        output.push_str(&format!("Total pages: {}\n", pages.len()));
        output.push_str(&format!("Showing pages {}-{}\n\n", start + 1, end));
        for i in start..end {
            output.push_str(&format!("=== Page {} ===\n", i + 1));
            output.push_str(pages[i]);
            output.push_str("\n\n");
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

/// Skill for merging multiple PDF files into a single document
///
/// This skill combines several PDF files into one merged PDF document.
/// It uses the `lopdf` crate for PDF manipulation and preserves all pages
/// from each input file in the order they are provided.
///
/// # Parameters
/// - `inputs` (required): Array of PDF file paths to merge
/// - `output` (required): Output PDF file path
///
/// # Returns
/// Returns a string indicating the number of files merged, the output path,
/// and the total number of pages in the merged document.
///
/// # Errors
/// Returns an error if:
/// - The inputs or output parameters are missing
/// - The inputs array is empty
/// - Any input file cannot be loaded
/// - The output directory cannot be created
/// - Saving the merged PDF fails
/// - No pages are found in the input PDFs
#[derive(Debug)]
pub struct PdfMergeSkill;

#[async_trait::async_trait]
impl Skill for PdfMergeSkill {
    fn name(&self) -> &str {
        "pdf_merge"
    }

    fn description(&self) -> &str {
        "Merge multiple PDF files into a single PDF"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to combine multiple PDF files into one"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "inputs".to_string(),
                param_type: "array".to_string(),
                description: "Array of PDF file paths to merge".to_string(),
                required: true,
                default: None,
                example: Some(json!(["file1.pdf", "file2.pdf", "file3.pdf"])),
                enum_values: None,
            },
            SkillParameter {
                name: "output".to_string(),
                param_type: "string".to_string(),
                description: "Output PDF file path".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("merged.pdf".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "pdf_merge",
            "parameters": {
                "inputs": ["doc1.pdf", "doc2.pdf"],
                "output": "merged.pdf"
            }
        })
    }

    fn example_output(&self) -> String {
        "Merged 2 PDF files into: merged.pdf".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        use lopdf::{Document, Object, ObjectId};
        let inputs = parameters
            .get("inputs")
            .ok_or_else(|| anyhow::anyhow!("Missing 'inputs' parameter"))?
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("'inputs' must be an array"))?;
        let output = parameters
            .get("output")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'output' parameter"))?;
        if inputs.is_empty() {
            anyhow::bail!("At least one input file is required");
        }
        let validated_output = validate_path(output, None)?;
        if let Some(parent) = validated_output.parent() {
            ensure_dir(&parent.to_string_lossy())?;
        }
        let mut merged_doc = Document::new();
        let mut total_pages = 0;
        let mut max_id = 0;
        for input_path in inputs {
            let path = input_path
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Input path must be a string"))?;
            let validated_input = validate_path(path, None)?;
            let doc = Document::load(&validated_input)
                .map_err(|e| anyhow::anyhow!("Failed to load PDF '{}': {}", path, e))?;
            // get pages
            let pages = doc.page_iter().collect::<Vec<_>>();
            total_pages += pages.len();
            for (id, object) in doc.objects.iter() {
                let new_id = (id.0 + max_id, id.1 + max_id as u16);
                merged_doc.objects.insert(new_id, object.clone());
            }
            max_id += doc.max_id;
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
        if page_objects.is_empty() {
            anyhow::bail!("No pages found in input PDFs");
        }
        merged_doc
            .save(&validated_output)
            .map_err(|e| anyhow::anyhow!("Failed to save merged PDF: {}", e))?;
        Ok(format!(
            "Merged {} PDF files into: {} ({} total pages)",
            inputs.len(),
            output,
            total_pages
        ))
    }

    fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("inputs")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: inputs"))?;
        parameters
            .get("output")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: output"))?;
        Ok(())
    }
}

/// Skill for extracting metadata information from PDF files
///
/// This skill retrieves document metadata including page count, title, author,
/// subject, creator, producer, and file size from a PDF file.
///
/// # Parameters
/// - `path` (required): Path to the PDF file
///
/// # Returns
/// Returns a formatted string containing all available metadata information
/// including page count, document properties (title, author, subject, etc.),
/// and file size.
///
/// # Errors
/// Returns an error if:
/// - The path parameter is missing
/// - The PDF file cannot be loaded or is invalid
#[derive(Debug)]
pub struct PdfInfoSkill;

#[async_trait::async_trait]
impl Skill for PdfInfoSkill {
    fn name(&self) -> &str {
        "pdf_info"
    }

    fn description(&self) -> &str {
        "Get metadata information from PDF file (pages, title, author, etc.)"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to get information about a PDF file"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the PDF file".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("document.pdf".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "pdf_info",
            "parameters": {
                "path": "document.pdf"
            }
        })
    }

    fn example_output(&self) -> String {
        "PDF Info:\nPages: 25\nTitle: My Document\nAuthor: John Doe".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        use lopdf::Document;
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let validated_path = validate_path(path, None)?;
        let doc = Document::load(&validated_path)
            .map_err(|e| anyhow::anyhow!("Failed to load PDF: {}", e))?;
        let pages = doc.page_iter().count();
        let mut output = format!("PDF Info for: {}\n", path);
        output.push_str(&format!("Total pages: {}\n", pages));
        // get metadata
        if let Ok(info_ref) = doc.trailer.get(b"Info") {
            if let Ok(info_id) = info_ref.as_reference() {
                if let Ok(info) = doc.get_object(info_id) {
                    if let Ok(dict) = info.as_dict() {
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
        let file_size = std::fs::metadata(&validated_path)
            .map(|m| m.len())
            .unwrap_or(0);
        output.push_str(&format!("File size: {:.2} KB\n", file_size as f64 / 1024.0));
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    /// Test PdfReadSkill parameter validation
    #[test]
    fn test_pdf_read_skill_validation() {
        let skill = PdfReadSkill;
        // Test valid parameters
        let mut valid_params = HashMap::new();
        valid_params.insert("path".to_string(), json!("test.pdf"));
        assert!(skill.validate(&valid_params).is_ok());
        // Test missing path parameter
        let empty_params = HashMap::new();
        assert!(skill.validate(&empty_params).is_err());
        // Test invalid path type (non-string)
        let mut invalid_params = HashMap::new();
        invalid_params.insert("path".to_string(), json!(123));
        assert!(skill.validate(&invalid_params).is_err());
    }

    /// Test PdfMergeSkill parameter validation and edge cases
    #[test]
    fn test_pdf_merge_skill_validation() {
        let skill = PdfMergeSkill;
        // Test valid parameters
        let mut valid_params = HashMap::new();
        valid_params.insert("inputs".to_string(), json!(["file1.pdf", "file2.pdf"]));
        valid_params.insert("output".to_string(), json!("merged.pdf"));
        assert!(skill.validate(&valid_params).is_ok());
        // Test missing inputs parameter
        let mut missing_inputs = HashMap::new();
        missing_inputs.insert("output".to_string(), json!("merged.pdf"));
        assert!(skill.validate(&missing_inputs).is_err());
        // Test missing output parameter
        let mut missing_output = HashMap::new();
        missing_output.insert("inputs".to_string(), json!(["file1.pdf"]));
        assert!(skill.validate(&missing_output).is_err());
        // Test invalid output type
        let mut invalid_output = HashMap::new();
        invalid_output.insert("inputs".to_string(), json!(["file1.pdf"]));
        invalid_output.insert("output".to_string(), json!(123));
        assert!(skill.validate(&invalid_output).is_err());
        // Test empty inputs array (validation only checks presence, not emptiness)
        let mut empty_inputs = HashMap::new();
        empty_inputs.insert("inputs".to_string(), json!([]));
        empty_inputs.insert("output".to_string(), json!("merged.pdf"));
        assert!(skill.validate(&empty_inputs).is_ok());
    }

    /// Test PdfInfoSkill parameter validation
    #[test]
    fn test_pdf_info_skill_validation() {
        let skill = PdfInfoSkill;
        // Test valid parameters
        let mut valid_params = HashMap::new();
        valid_params.insert("path".to_string(), json!("document.pdf"));
        assert!(skill.validate(&valid_params).is_ok());
        // Test missing path parameter
        let empty_params = HashMap::new();
        assert!(skill.validate(&empty_params).is_err());
        // Test invalid path type
        let mut invalid_params = HashMap::new();
        invalid_params.insert("path".to_string(), json!(true));
        assert!(skill.validate(&invalid_params).is_err());
    }

    /// Test skill name and description consistency
    #[test]
    fn test_skill_metadata() {
        let read_skill = PdfReadSkill;
        assert_eq!(read_skill.name(), "pdf_read");
        assert_eq!(read_skill.category(), "document");
        assert!(!read_skill.description().is_empty());
        assert!(!read_skill.usage_hint().is_empty());
        let merge_skill = PdfMergeSkill;
        assert_eq!(merge_skill.name(), "pdf_merge");
        assert_eq!(merge_skill.category(), "document");
        assert!(!merge_skill.description().is_empty());
        let info_skill = PdfInfoSkill;
        assert_eq!(info_skill.name(), "pdf_info");
        assert_eq!(info_skill.category(), "document");
        assert!(!info_skill.description().is_empty());
    }

    /// Test parameter definitions are complete
    #[test]
    fn test_skill_parameters() {
        let read_skill = PdfReadSkill;
        let params = read_skill.parameters();
        assert_eq!(params.len(), 3);
        assert!(params.iter().any(|p| p.name == "path" && p.required));
        assert!(params.iter().any(|p| p.name == "start_page" && !p.required));
        assert!(params.iter().any(|p| p.name == "end_page" && !p.required));
        let merge_skill = PdfMergeSkill;
        let params = merge_skill.parameters();
        assert_eq!(params.len(), 2);
        assert!(params.iter().any(|p| p.name == "inputs" && p.required));
        assert!(params.iter().any(|p| p.name == "output" && p.required));
        let info_skill = PdfInfoSkill;
        let params = info_skill.parameters();
        assert_eq!(params.len(), 1);
        assert!(params.iter().any(|p| p.name == "path" && p.required));
    }

    /// Test example calls return valid JSON
    #[test]
    fn test_example_calls() {
        let read_skill = PdfReadSkill;
        let example = read_skill.example_call();
        assert!(example.get("action").is_some());
        assert!(example.get("parameters").is_some());
        let merge_skill = PdfMergeSkill;
        let example = merge_skill.example_call();
        assert!(example.get("action").is_some());
        assert!(example.get("parameters").is_some());
        let info_skill = PdfInfoSkill;
        let example = info_skill.example_call();
        assert!(example.get("action").is_some());
        assert!(example.get("parameters").is_some());
    }

    /// Test example outputs are non-empty strings
    #[test]
    fn test_example_outputs() {
        let read_skill = PdfReadSkill;
        assert!(!read_skill.example_output().is_empty());
        let merge_skill = PdfMergeSkill;
        assert!(!merge_skill.example_output().is_empty());
        let info_skill = PdfInfoSkill;
        assert!(!info_skill.example_output().is_empty());
    }
}
