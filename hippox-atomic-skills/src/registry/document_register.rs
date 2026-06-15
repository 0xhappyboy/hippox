//! Document processing skills registration

use std::collections::HashMap;
use std::sync::Arc;

use crate::{SkillCategory, SkillRegistryMap};

pub fn register(registry: &mut SkillRegistryMap) {
    let category = SkillCategory::Document;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "document", feature = "all"))]
    {
        use crate::skills::document::*;
        
        map.insert("markdown_read".to_string(), Arc::new(MarkdownReadSkill));
        map.insert("markdown_write".to_string(), Arc::new(MarkdownWriteSkill));
        map.insert("csv_read".to_string(), Arc::new(CsvReadSkill));
        map.insert("csv_write".to_string(), Arc::new(CsvWriteSkill));
        map.insert("xml_parse".to_string(), Arc::new(XmlParseSkill));
        map.insert("xml_to_json".to_string(), Arc::new(XmlToJsonSkill));
        map.insert("excel_read".to_string(), Arc::new(ExcelReadSkill));
        map.insert("excel_write".to_string(), Arc::new(ExcelWriteSkill));
        map.insert("pdf_read".to_string(), Arc::new(PdfReadSkill));
        map.insert("pdf_merge".to_string(), Arc::new(PdfMergeSkill));
        map.insert("pdf_info".to_string(), Arc::new(PdfInfoSkill));
        map.insert("json_read".to_string(), Arc::new(JsonReadSkill));
        map.insert("json_write".to_string(), Arc::new(JsonWriteSkill));
        map.insert("json_validate".to_string(), Arc::new(JsonValidateSkill));
        map.insert("yaml_read".to_string(), Arc::new(YamlReadSkill));
        map.insert("yaml_write".to_string(), Arc::new(YamlWriteSkill));
        map.insert("yaml_validate".to_string(), Arc::new(YamlValidateSkill));
        map.insert("toml_read".to_string(), Arc::new(TomlReadSkill));
        map.insert("toml_write".to_string(), Arc::new(TomlWriteSkill));
        map.insert("toml_validate".to_string(), Arc::new(TomlValidateSkill));
        map.insert("text_read".to_string(), Arc::new(TextReadSkill));
        map.insert("text_write".to_string(), Arc::new(TextWriteSkill));
        map.insert("text_search".to_string(), Arc::new(TextSearchSkill));
        map.insert("html_read".to_string(), Arc::new(HtmlReadSkill));
        map.insert("html_write".to_string(), Arc::new(HtmlWriteSkill));
        map.insert("html_validate".to_string(), Arc::new(HtmlValidateSkill));
        map.insert("pptx_read".to_string(), Arc::new(PptxReadSkill));
        map.insert("pptx_info".to_string(), Arc::new(PptxInfoSkill));
        map.insert("docx_read".to_string(), Arc::new(DocxReadSkill));
        map.insert("docx_info".to_string(), Arc::new(DocxInfoSkill));
        map.insert("ods_read".to_string(), Arc::new(OdsReadSkill));
        map.insert("odt_read".to_string(), Arc::new(OdtReadSkill));
    }
}