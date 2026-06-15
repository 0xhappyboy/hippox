use anyhow::Result;
use quick_xml::{Reader, events::Event};
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::executors::{
    ensure_dir, file_exists, read_file_content,
    types::{Skill, SkillParameter},
    validate_path, write_file_content,
};

#[derive(Debug)]
pub struct PptxReadSkill;

#[async_trait::async_trait]
impl Skill for PptxReadSkill {
    fn name(&self) -> &str {
        "pptx_read"
    }

    fn description(&self) -> &str {
        "Read and extract text content from PowerPoint (.pptx) files"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to read PowerPoint presentations, extract slide content, or convert PPTX to text"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the PPTX file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("presentation.pptx".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "slide_number".to_string(),
                param_type: "integer".to_string(),
                description: "Specific slide number to extract (1-indexed)".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(1.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "include_notes".to_string(),
                param_type: "boolean".to_string(),
                description: "Include speaker notes".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "pptx_read",
            "parameters": {
                "path": "presentation.pptx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Slide 1: Title\nSlide 2: Content...".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        let specific_slide = parameters
            .get("slide_number")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        let include_notes = parameters
            .get("include_notes")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("PPTX file not found: {}", path);
        }
        use quick_xml::Reader;
        use quick_xml::events::Event;
        use std::fs::File;
        use zip::ZipArchive;
        let file = File::open(&validated_path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut slides: Vec<(usize, String)> = Vec::new();
        let mut notes: HashMap<usize, String> = HashMap::new();
        for i in 0..archive.len() {
            let entry = archive.by_index(i)?;
            let name = entry.name().to_string();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                let slide_num = extract_slide_number(&name);
                let content = read_zip_entry_text(entry)?;
                let text = extract_text_from_xml(&content);
                slides.push((slide_num, text));
            } else if include_notes
                && name.starts_with("ppt/notesSlides/notesSlide")
                && name.ends_with(".xml")
            {
                let slide_num = extract_slide_number(&name);
                let content = read_zip_entry_text(entry)?;
                let text = extract_text_from_xml(&content);
                notes.insert(slide_num, text);
            }
        }
        slides.sort_by_key(|(num, _)| *num);
        let mut output = String::new();
        let slides_len = slides.len();
        if let Some(slide_num) = specific_slide {
            if let Some((_, content)) = slides.iter().find(|(num, _)| *num == slide_num) {
                output.push_str(&format!("Slide {}:\n{}\n", slide_num, content));
                if include_notes {
                    if let Some(note_text) = notes.get(&slide_num) {
                        output.push_str(&format!("Notes: {}\n", note_text));
                    }
                }
            } else {
                anyhow::bail!("Slide {} not found", slide_num);
            }
        } else {
            for (slide_num, content) in slides {
                output.push_str(&format!("Slide {}:\n{}\n\n", slide_num, content));
                if include_notes {
                    if let Some(note_text) = notes.get(&slide_num) {
                        output.push_str(&format!("Notes: {}\n\n", note_text));
                    }
                }
            }
        }
        output.push_str(&format!("Total slides: {}", slides_len));
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
pub struct PptxInfoSkill;

#[async_trait::async_trait]
impl Skill for PptxInfoSkill {
    fn name(&self) -> &str {
        "pptx_info"
    }

    fn description(&self) -> &str {
        "Get metadata and structure information about a PowerPoint file"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to get slide count, file info, or presentation metadata"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![SkillParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the PPTX file".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("presentation.pptx".to_string())),
            enum_values: None,
        }]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "pptx_info",
            "parameters": {
                "path": "presentation.pptx"
            }
        })
    }

    fn example_output(&self) -> String {
        "Slides: 10\nFile size: 1.2 MB\nCreated: 2024-01-01".to_string()
    }

    fn category(&self) -> &str {
        "document"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let path = parameters
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        let validated_path = validate_path(path, None)?;
        if !file_exists(&validated_path.to_string_lossy()) {
            anyhow::bail!("PPTX file not found: {}", path);
        }

        use std::fs::File;
        use zip::ZipArchive;

        let file = File::open(&validated_path)?;
        let mut archive = ZipArchive::new(file)?;

        let metadata = std::fs::metadata(&validated_path)?;
        let file_size = metadata.len();

        let mut slide_count = 0;
        let mut has_notes = false;

        for i in 0..archive.len() {
            let entry = archive.by_index(i)?;
            let name = entry.name();

            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                slide_count += 1;
            } else if name.starts_with("ppt/notesSlides/") {
                has_notes = true;
            }
        }
        let mut output = String::new();
        output.push_str(&format!("File: {}\n", path));
        output.push_str(&format!("File size: {:.2} KB\n", file_size as f64 / 1024.0));
        output.push_str(&format!("Number of slides: {}\n", slide_count));
        output.push_str(&format!(
            "Contains speaker notes: {}\n",
            if has_notes { "Yes" } else { "No" }
        ));
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

fn extract_slide_number(filename: &str) -> usize {
    let parts: Vec<&str> = filename.split('/').collect();
    if let Some(last) = parts.last() {
        let num_str = last
            .replace("slide", "")
            .replace(".xml", "")
            .replace("notesSlide", "");
        num_str.parse().unwrap_or(0)
    } else {
        0
    }
}

fn read_zip_entry_text<R: std::io::Read + std::io::Seek>(
    mut entry: zip::read::ZipFile<'_, R>,
) -> Result<String> {
    let mut content = String::new();
    std::io::Read::read_to_string(&mut entry, &mut content)?;
    Ok(content)
}

fn extract_text_from_xml(xml: &str) -> String {
    let mut reader = Reader::from_str(xml);
    let mut text_parts = Vec::new();
    let mut in_text = false;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"a:t" || e.name().as_ref() == b"t" {
                    in_text = true;
                }
            }
            Ok(Event::Text(e)) => {
                if in_text {
                    if let Ok(text) = e.decode() {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            text_parts.push(trimmed.to_string());
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"a:t" || e.name().as_ref() == b"t" {
                    in_text = false;
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
