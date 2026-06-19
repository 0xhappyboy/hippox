//! Document processing drivers registration

use std::collections::HashMap;
use std::sync::Arc;

use crate::{DriverCategory, DriverRegistryMap};

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Document;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "document", feature = "all"))]
    {
        use crate::drivers::document::*;
        
        map.insert("markdown_read".to_string(), Arc::new(MarkdownReadDriver));
        map.insert("markdown_write".to_string(), Arc::new(MarkdownWriteDriver));
        map.insert("csv_read".to_string(), Arc::new(CsvReadDriver));
        map.insert("csv_write".to_string(), Arc::new(CsvWriteDriver));
        map.insert("xml_parse".to_string(), Arc::new(XmlParseDriver));
        map.insert("xml_to_json".to_string(), Arc::new(XmlToJsonDriver));
        map.insert("excel_read".to_string(), Arc::new(ExcelReadDriver));
        map.insert("excel_write".to_string(), Arc::new(ExcelWriteDriver));
        map.insert("pdf_read".to_string(), Arc::new(PdfReadDriver));
        map.insert("pdf_merge".to_string(), Arc::new(PdfMergeDriver));
        map.insert("pdf_info".to_string(), Arc::new(PdfInfoDriver));
        map.insert("json_read".to_string(), Arc::new(JsonReadDriver));
        map.insert("json_write".to_string(), Arc::new(JsonWriteDriver));
        map.insert("json_validate".to_string(), Arc::new(JsonValidateDriver));
        map.insert("yaml_read".to_string(), Arc::new(YamlReadDriver));
        map.insert("yaml_write".to_string(), Arc::new(YamlWriteDriver));
        map.insert("yaml_validate".to_string(), Arc::new(YamlValidateDriver));
        map.insert("toml_read".to_string(), Arc::new(TomlReadDriver));
        map.insert("toml_write".to_string(), Arc::new(TomlWriteDriver));
        map.insert("toml_validate".to_string(), Arc::new(TomlValidateDriver));
        map.insert("text_read".to_string(), Arc::new(TextReadDriver));
        map.insert("text_write".to_string(), Arc::new(TextWriteDriver));
        map.insert("text_search".to_string(), Arc::new(TextSearchDriver));
        map.insert("html_read".to_string(), Arc::new(HtmlReadDriver));
        map.insert("html_write".to_string(), Arc::new(HtmlWriteDriver));
        map.insert("html_validate".to_string(), Arc::new(HtmlValidateDriver));
        map.insert("pptx_read".to_string(), Arc::new(PptxReadDriver));
        map.insert("pptx_info".to_string(), Arc::new(PptxInfoDriver));
        map.insert("docx_read".to_string(), Arc::new(DocxReadDriver));
        map.insert("docx_info".to_string(), Arc::new(DocxInfoDriver));
        map.insert("ods_read".to_string(), Arc::new(OdsReadDriver));
        map.insert("odt_read".to_string(), Arc::new(OdtReadDriver));
    }
}