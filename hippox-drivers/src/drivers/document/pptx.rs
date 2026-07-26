//! PowerPoint file driver module
//!
//! This module provides drivers for PowerPoint PPTX file operations including
//! reading presentation content, extracting slide text, and getting
//! presentation metadata.
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
/// Driver for reading PPTX files and extracting slide content
#[derive(Debug)]
pub struct PptxReadDriver;
#[async_trait::async_trait]
impl Driver for PptxReadDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "pptx_read";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Read and extract text content from PowerPoint (.pptx) files";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to read PowerPoint presentations, extract slide content, or convert PPTX to text";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "path".to_string(),
                param_type: "string".to_string(),
                description: "Path to the PPTX file".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("presentation.pptx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "slide_number".to_string(),
                param_type: "integer".to_string(),
                description: "Specific slide number to extract (1-indexed)".to_string(),
                required: false,
                default: None,
                example: Some(Value::Number(1.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "include_notes".to_string(),
                param_type: "boolean".to_string(),
                description: "Include speaker notes".to_string(),
                required: false,
                default: Some(Value::Bool(false)),
                example: Some(Value::Bool(true)),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "pptx_read",
            "parameters": {
                "path": "presentation.pptx"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Slide 1: Title\nSlide 2: Content...".to_string();
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
        debug!("Executing pptx_read driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        let specific_slide = parameters.get("slide_number").and_then(|v| v.as_u64()).map(|v| v as usize);
        let include_notes = parameters.get("include_notes").and_then(|v| v.as_bool()).unwrap_or(false);
        debug!("Reading PPTX file: {}, slide: {:?}, include_notes: {}", path, specific_slide, include_notes);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("PPTX file not found: {}", path)));
        }
        use std::fs::File;
        use zip::ZipArchive;
        let file = File::open(&validated_path).map_err(|e| DriverError::execution(format!("Failed to open file: {}", e)))?;
        let mut archive = ZipArchive::new(file).map_err(|e| DriverError::execution(format!("Failed to open PPTX archive: {}", e)))?;
        let mut slides: Vec<(usize, String)> = Vec::new();
        let mut notes: HashMap<usize, String> = HashMap::new();
        for i in 0..archive.len() {
            let entry = archive.by_index(i).map_err(|e| DriverError::execution(format!("Failed to read archive entry: {}", e)))?;
            let name = entry.name().to_string();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                let slide_num = extract_slide_number(&name);
                let content = read_zip_entry_text(entry)?;
                let text = extract_text_from_xml(&content);
                slides.push((slide_num, text));
            } else if include_notes && name.starts_with("ppt/notesSlides/notesSlide") && name.ends_with(".xml") {
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
                return Err(DriverError::execution(format!("Slide {} not found", slide_num)));
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
        info!("PPTX read completed: {} ({} slides)", path, slides_len);
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Driver for getting PPTX file metadata and structure information
#[derive(Debug)]
pub struct PptxInfoDriver;
#[async_trait::async_trait]
impl Driver for PptxInfoDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        return "pptx_info";
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        return "Get metadata and structure information about a PowerPoint file";
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        return "Use this skill when the user wants to get slide count, file info, or presentation metadata";
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![DriverParameter {
            name: "path".to_string(),
            param_type: "string".to_string(),
            description: "Path to the PPTX file".to_string(),
            required: true,
            default: None,
            example: Some(Value::String("presentation.pptx".to_string())),
            enum_values: None,
        }];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "pptx_info",
            "parameters": {
                "path": "presentation.pptx"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Slides: 10\nFile size: 1.2 MB\nCreated: 2024-01-01".to_string();
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
        debug!("Executing pptx_info driver");
        // Extract required parameters
        let path = parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        debug!("Getting PPTX info for: {}", path);
        let validated_path = validate_path(path, None).map_err(|e| DriverError::execution(format!("Invalid path: {}", e)))?;
        if !file_exists(&validated_path.to_string_lossy()) {
            return Err(DriverError::execution(format!("PPTX file not found: {}", path)));
        }
        use std::fs::File;
        use zip::ZipArchive;
        let file = File::open(&validated_path).map_err(|e| DriverError::execution(format!("Failed to open file: {}", e)))?;
        let mut archive = ZipArchive::new(file).map_err(|e| DriverError::execution(format!("Failed to open PPTX archive: {}", e)))?;
        let metadata = std::fs::metadata(&validated_path).map_err(|e| DriverError::execution(format!("Failed to read file metadata: {}", e)))?;
        let file_size = metadata.len();
        let mut slide_count = 0;
        let mut has_notes = false;
        for i in 0..archive.len() {
            let entry = archive.by_index(i).map_err(|e| DriverError::execution(format!("Failed to read archive entry: {}", e)))?;
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
        output.push_str(&format!("Contains speaker notes: {}\n", if has_notes { "Yes" } else { "No" }));
        info!("PPTX info completed: {} ({} slides)", path, slide_count);
        return Ok(output);
    }
    /// Validates the parameters before execution
    fn validate(&self, parameters: &HashMap<String, Value>) -> DriverResult<()> {
        parameters.get("path").and_then(|v| v.as_str()).ok_or_else(|| DriverError::missing_parameter("path"))?;
        return Ok(());
    }
}
/// Extracts the slide number from a filename
///
/// # Arguments
/// * `filename` - Filename containing slide number
///
/// # Returns
/// * `usize` - Extracted slide number
fn extract_slide_number(filename: &str) -> usize {
    let parts: Vec<&str> = filename.split('/').collect();
    if let Some(last) = parts.last() {
        let num_str = last.replace("slide", "").replace(".xml", "").replace("notesSlide", "");
        return num_str.parse().unwrap_or(0);
    }
    return 0;
}
/// Reads text content from a zip entry
///
/// # Arguments
/// * `entry` - Zip file entry
///
/// # Returns
/// * `DriverResult<String>` - Entry content
fn read_zip_entry_text<R: std::io::Read + std::io::Seek>(mut entry: zip::read::ZipFile<'_, R>) -> DriverResult<String> {
    let mut content = String::new();
    std::io::Read::read_to_string(&mut entry, &mut content).map_err(|e| DriverError::execution(format!("Failed to read zip entry: {}", e)))?;
    return Ok(content);
}
/// Extracts text content from XML
///
/// # Arguments
/// * `xml` - XML content
///
/// # Returns
/// * `String` - Extracted text
fn extract_text_from_xml(xml: &str) -> String {
    use quick_xml::{Reader, events::Event};
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
                debug!("Error parsing XML: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }
    return text_parts.join(" ");
}
