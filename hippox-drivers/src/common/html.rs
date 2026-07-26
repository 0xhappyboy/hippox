//! HTML parsing utilities
//!
//! This module provides functionality for parsing and extracting
//! information from HTML documents using the scraper crate.
use crate::DriverError;
use crate::result::DriverResult;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
/// HTML parse result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HtmlParseResult {
    pub title: Option<String>,
    pub links: Vec<String>,
    pub images: Vec<String>,
    pub headings: Vec<String>,
    pub paragraphs: Vec<String>,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub all_text: String,
}
/// Parse HTML content and extract structured information
///
/// # Arguments
/// * `html` - HTML content as string
/// * `extract_all` - If true, extract all elements; if false, only extract title
///
/// # Returns
/// `DriverResult<HtmlParseResult>` containing extracted data
pub fn parse_html(html: &str, extract_all: bool) -> DriverResult<HtmlParseResult> {
    debug!("Parsing HTML content, length: {}, extract_all: {}", html.len(), extract_all);
    let document = Html::parse_document(html);
    let mut result = HtmlParseResult {
        title: None,
        links: Vec::new(),
        images: Vec::new(),
        headings: Vec::new(),
        paragraphs: Vec::new(),
        meta_description: None,
        meta_keywords: None,
        all_text: String::new(),
    };
    // If not extracting all, only get title
    if !extract_all {
        if let Ok(selector) = Selector::parse("title") {
            if let Some(elem) = document.select(&selector).next() {
                result.title = Some(elem.text().collect::<Vec<_>>().concat());
                info!("Title extracted from HTML: {:?}", result.title);
            }
        }
        return Ok(result);
    }
    // Extract title
    if let Ok(selector) = Selector::parse("title") {
        if let Some(elem) = document.select(&selector).next() {
            result.title = Some(elem.text().collect::<Vec<_>>().concat());
            debug!("Title: {:?}", result.title);
        }
    }
    // Extract links
    if let Ok(selector) = Selector::parse("a[href]") {
        for elem in document.select(&selector) {
            if let Some(href) = elem.value().attr("href") {
                result.links.push(href.to_string());
            }
        }
        debug!("Extracted {} links", result.links.len());
    }
    // Extract images
    if let Ok(selector) = Selector::parse("img[src]") {
        for elem in document.select(&selector) {
            if let Some(src) = elem.value().attr("src") {
                result.images.push(src.to_string());
            }
        }
        debug!("Extracted {} images", result.images.len());
    }
    // Extract headings (h1 through h6)
    for level in 1..=6 {
        if let Ok(selector) = Selector::parse(&format!("h{}", level)) {
            for elem in document.select(&selector) {
                let text = elem.text().collect::<Vec<_>>().concat();
                if !text.is_empty() {
                    result.headings.push(format!("h{}: {}", level, text));
                }
            }
        }
    }
    debug!("Extracted {} headings", result.headings.len());
    // Extract paragraphs
    if let Ok(selector) = Selector::parse("p") {
        for elem in document.select(&selector) {
            let text = elem.text().collect::<Vec<_>>().concat();
            if !text.is_empty() {
                result.paragraphs.push(text);
            }
        }
        debug!("Extracted {} paragraphs", result.paragraphs.len());
    }
    // Extract meta description
    if let Ok(selector) = Selector::parse("meta[name='description']") {
        for elem in document.select(&selector) {
            if let Some(content) = elem.value().attr("content") {
                result.meta_description = Some(content.to_string());
                debug!("Meta description: {:?}", result.meta_description);
                break;
            }
        }
    }
    // Extract meta keywords
    if let Ok(selector) = Selector::parse("meta[name='keywords']") {
        for elem in document.select(&selector) {
            if let Some(content) = elem.value().attr("content") {
                result.meta_keywords = Some(content.to_string());
                debug!("Meta keywords: {:?}", result.meta_keywords);
                break;
            }
        }
    }
    // Extract all text from body
    if let Ok(selector) = Selector::parse("body") {
        if let Some(body) = document.select(&selector).next() {
            let text = body.text().collect::<Vec<_>>().concat();
            result.all_text = text;
            debug!("Extracted {} bytes of text from body", result.all_text.len());
        }
    }
    info!("HTML parsing completed: title={:?}, links={}, images={}", result.title, result.links.len(), result.images.len());
    return Ok(result);
}
