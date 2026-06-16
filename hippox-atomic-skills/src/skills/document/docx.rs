use anyhow::Result;
use quick_xml::{Reader, events::Event};
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};
use crate::{ensure_dir, file_exists, read_file_content, validate_path, write_file_content};

#[derive(Debug)]
pub struct DocxReadSkill;

#[async_trait::async_trait]
impl Skill for DocxReadSkill {
    fn name(&self) -> &str {
        "docx_read"
    }

    fn description(&self) -> &str {
        "Read and extract text content from Word (.docx) files"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read Microsoft Word documents, extract text, or convert DOCX to plain text"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the DOCX file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("document.docx".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "include_tables".to_string(),
                param_type: "boolean".to_string(),
                description: "Include table data in output".to_string(),
                required: false,
                default: Some(Value::Bool(true)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "docx_read",
            "parameters": {
                "path": "document.docx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Document content extracted from Word file...".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Document
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let include_tables = parameters
            .get("include_tables")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("DOCX file not found: {}", path);
        }

        use std::fs::File;
        use zip::ZipArchive;
        let file = File::open(&validated_path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut document_content = None;
        for i in 0..archive.len() {
            let entry = archive.by_index(i)?;
            if entry.name() == "word/document.xml" {
                let mut content = String::new();
                let mut reader = std::io::BufReader::new(entry);
                std::io::Read::read_to_string(&mut reader, &mut content)?;
                document_content = Some(content);
                break;
            }
        }
        let content = document_content
            .ok_or_else(|| anyhow::anyhow!("No document.xml found in DOCX file"))?;
        let text = extract_text_from_docx_xml(&content, include_tables);
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

#[derive(Debug)]
pub struct DocxInfoSkill;

#[async_trait::async_trait]
impl Skill for DocxInfoSkill {
    fn name(&self) -> &str {
        "docx_info"
    }

    fn description(&self) -> &str {
        "Get metadata and structure information about a Word document"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to get document properties, word count, or file info"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the DOCX file".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("document.docx".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "docx_info",
            "parameters": {
                "path": "document.docx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Word count: 1500\nPages: 5\nFile size: 120 KB".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Document
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("DOCX file not found: {}", path);
        }
        use std::fs::File;
        use zip::ZipArchive;
        let file = File::open(&validated_path)?;
        let mut archive = ZipArchive::new(file)?;
        let metadata = std::fs::metadata(&validated_path)?;
        let file_size = metadata.len();
        let mut document_content = None;
        for i in 0..archive.len() {
            let entry = archive.by_index(i)?;
            if entry.name() == "word/document.xml" {
                let mut content = String::new();
                let mut reader = std::io::BufReader::new(entry);
                std::io::Read::read_to_string(&mut reader, &mut content)?;
                document_content = Some(content);
                break;
            }
        }
        let mut output = String::new();
        output.push_str(&format!("File: {}\n", path));
        output.push_str(&format!("File size: {:.2} KB\n", file_size as f64 / 1024.0));
        if let Some(content) = document_content {
            let text = extract_text_from_docx_xml(&content, false);
            let word_count = text.split_whitespace().count();
            let char_count = text.chars().count();
            let line_count = text.lines().count();
            output.push_str(&format!("Word count: {}\n", word_count));
            output.push_str(&format!("Character count: {}\n", char_count));
            output.push_str(&format!("Line count: {}\n", line_count));
        } else {
            output.push_str("Unable to extract document content\n");
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

fn extract_text_from_docx_xml(xml: &str, include_tables: bool) -> String {
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
                eprintln!("Error parsing XML: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }
    text_parts.join(" ")
}

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
    output
}
