use crate::SkillCallback;
use crate::SkillContext;
use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use crate::{ensure_dir, file_exists, validate_path};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

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

    fn category(&self) -> SkillCategory {
        SkillCategory::Document
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let skill_index = context.as_ref().and_then(|c| c.skill_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.skill_name())
            .map(String::from);
        let cb = callback;

        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), skill_index, step_name);
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Starting PDF read operation".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(10), None);
        }

        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Reading PDF file: {}", path)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }

        let start_page = parameters
            .get("start_page")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;
        let end_page = parameters
            .get("end_page")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!(
                    "Start page: {}, End page: {:?}",
                    start_page, end_page
                )),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(30), None);
        }

        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("PDF file not found: {}", path);
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Validated path, loading PDF".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(40), None);
        }

        use pdf_extract::extract_text;
        let full_text = extract_text(&validated_path)
            .map_err(|e| anyhow::anyhow!("Failed to extract PDF text: {}", e))?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("PDF text extracted successfully".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(60), None);
        }

        let pages: Vec<&str> = full_text.split("\n\n").collect();
        let start = start_page.saturating_sub(1);
        let end = end_page.unwrap_or(pages.len()).min(pages.len());

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!(
                    "Total pages: {}, Selected range: {}-{}",
                    pages.len(),
                    start + 1,
                    end
                )),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(75), None);
        }

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

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!(
                    "Completed PDF read, output length: {} bytes",
                    output.len()
                )),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                skill_index,
                Some("pdf_read".to_string()),
                Some(output.clone()),
            );
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

    fn category(&self) -> SkillCategory {
        SkillCategory::Document
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let skill_index = context.as_ref().and_then(|c| c.skill_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.skill_name())
            .map(String::from);
        let cb = callback;

        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), skill_index, step_name);
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Starting PDF merge operation".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(10), None);
        }
        use lopdf::{Document, Object, ObjectId};
        let inputs = parameters
            .get("inputs")
            .ok_or_else(|| anyhow::anyhow!("Missing 'inputs' parameter"))?
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("'inputs' must be an array"))?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Number of input files: {}", inputs.len())),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(15), None);
        }

        let output = parameters
            .get("output")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'output' parameter"))?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Output file: {}", output)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }

        if inputs.is_empty() {
            anyhow::bail!("At least one input file is required");
        }

        let validated_output = validate_path(output, None)?;
        if let Some(parent) = validated_output.parent() {
            ensure_dir(&parent.to_string_lossy())?;
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Validated output path, creating directory".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(25), None);
        }

        let mut merged_doc = Document::new();
        let mut total_pages = 0;
        let mut max_id = 0;
        let total_inputs = inputs.len();

        for (idx, input_path) in inputs.iter().enumerate() {
            let path = input_path
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("Input path must be a string"))?;

            if let Some(cb) = cb {
                let progress = 25 + ((idx + 1) as f32 / total_inputs as f32 * 50.0) as u8;
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some(format!(
                        "Processing file {}/{}: {}",
                        idx + 1,
                        total_inputs,
                        path
                    )),
                );
                cb.on_progress(task_id.clone(), skill_index, Some(progress), None);
            }

            let validated_input = validate_path(path, None)?;
            let doc = Document::load(&validated_input)
                .map_err(|e| anyhow::anyhow!("Failed to load PDF '{}': {}", path, e))?;

            let pages = doc.page_iter().collect::<Vec<_>>();
            total_pages += pages.len();

            if let Some(cb) = cb {
                cb.on_log(
                    task_id.clone(),
                    skill_index,
                    Some(format!("Loaded PDF with {} pages", pages.len())),
                );
            }

            for (id, object) in doc.objects.iter() {
                let new_id = (id.0 + max_id, id.1 + max_id as u16);
                merged_doc.objects.insert(new_id, object.clone());
            }
            max_id += doc.max_id;
        }

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("All files loaded, total pages: {}", total_pages)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(80), None);
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

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Found {} page objects", page_objects.len())),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(90), None);
        }

        if page_objects.is_empty() {
            anyhow::bail!("No pages found in input PDFs");
        }

        merged_doc
            .save(&validated_output)
            .map_err(|e| anyhow::anyhow!("Failed to save merged PDF: {}", e))?;

        let result = format!(
            "Merged {} PDF files into: {} ({} total pages)",
            inputs.len(),
            output,
            total_pages
        );

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Merge completed: {}", result)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                skill_index,
                Some("pdf_merge".to_string()),
                Some(result.clone()),
            );
        }

        Ok(result)
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

    fn category(&self) -> SkillCategory {
        SkillCategory::Document
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn SkillCallback>,
        context: Option<&SkillContext>,
    ) -> Result<String> {
        let task_id = context.as_ref().and_then(|c| c.task_id()).map(String::from);
        let skill_index = context.as_ref().and_then(|c| c.skill_index());
        let step_name = context
            .as_ref()
            .and_then(|c| c.skill_name())
            .map(String::from);
        let cb = callback;

        if let Some(cb) = cb {
            cb.on_start(task_id.clone(), skill_index, step_name);
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Starting PDF info operation".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(10), None);
        }

        use lopdf::Document;
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Reading PDF info for: {}", path)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(20), None);
        }

        let validated_path = validate_path(path, None)?;

        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("Validated path, loading PDF".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(30), None);
        }
        let doc = Document::load(&validated_path)
            .map_err(|e| anyhow::anyhow!("Failed to load PDF: {}", e))?;
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some("PDF loaded successfully".to_string()),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(50), None);
        }
        let pages = doc.page_iter().count();
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!("Total pages: {}", pages)),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(60), None);
        }
        let mut output = format!("PDF Info for: {}\n", path);
        output.push_str(&format!("Total pages: {}\n", pages));
        if let Ok(info_ref) = doc.trailer.get(b"Info") {
            if let Ok(info_id) = info_ref.as_reference() {
                if let Ok(info) = doc.get_object(info_id) {
                    if let Ok(dict) = info.as_dict() {
                        if let Some(cb) = cb {
                            cb.on_log(
                                task_id.clone(),
                                skill_index,
                                Some("Extracting metadata from PDF".to_string()),
                            );
                            cb.on_progress(task_id.clone(), skill_index, Some(70), None);
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
        let file_size = std::fs::metadata(&validated_path)
            .map(|m| m.len())
            .unwrap_or(0);
        output.push_str(&format!("File size: {:.2} KB\n", file_size as f64 / 1024.0));
        if let Some(cb) = cb {
            cb.on_log(
                task_id.clone(),
                skill_index,
                Some(format!(
                    "PDF info completed, output length: {} bytes",
                    output.len()
                )),
            );
            cb.on_progress(task_id.clone(), skill_index, Some(100), None);
            cb.on_complete(
                task_id.clone(),
                skill_index,
                Some("pdf_info".to_string()),
                Some(output.clone()),
            );
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
