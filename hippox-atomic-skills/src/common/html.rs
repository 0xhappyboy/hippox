//! HTML parsing utilities

use anyhow::Result;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

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

pub fn parse_html(html: &str, extract_all: bool) -> Result<HtmlParseResult> {
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

    if !extract_all {
        if let Ok(selector) = Selector::parse("title") {
            if let Some(elem) = document.select(&selector).next() {
                result.title = Some(elem.text().collect::<Vec<_>>().concat());
            }
        }
        return Ok(result);
    }

    if let Ok(selector) = Selector::parse("title") {
        if let Some(elem) = document.select(&selector).next() {
            result.title = Some(elem.text().collect::<Vec<_>>().concat());
        }
    }

    if let Ok(selector) = Selector::parse("a[href]") {
        for elem in document.select(&selector) {
            if let Some(href) = elem.value().attr("href") {
                result.links.push(href.to_string());
            }
        }
    }

    if let Ok(selector) = Selector::parse("img[src]") {
        for elem in document.select(&selector) {
            if let Some(src) = elem.value().attr("src") {
                result.images.push(src.to_string());
            }
        }
    }

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

    if let Ok(selector) = Selector::parse("p") {
        for elem in document.select(&selector) {
            let text = elem.text().collect::<Vec<_>>().concat();
            if !text.is_empty() {
                result.paragraphs.push(text);
            }
        }
    }

    if let Ok(selector) = Selector::parse("meta[name='description']") {
        for elem in document.select(&selector) {
            if let Some(content) = elem.value().attr("content") {
                result.meta_description = Some(content.to_string());
                break;
            }
        }
    }

    if let Ok(selector) = Selector::parse("meta[name='keywords']") {
        for elem in document.select(&selector) {
            if let Some(content) = elem.value().attr("content") {
                result.meta_keywords = Some(content.to_string());
                break;
            }
        }
    }

    if let Ok(selector) = Selector::parse("body") {
        if let Some(body) = document.select(&selector).next() {
            let text = body.text().collect::<Vec<_>>().concat();
            result.all_text = text;
        }
    }

    Ok(result)
}
