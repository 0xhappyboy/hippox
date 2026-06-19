//! Text processing drivers registration

use std::collections::HashMap;
use std::sync::Arc;
use crate::{DriverCategory, DriverRegistryMap};

pub fn register(registry: &mut DriverRegistryMap) {
    let category = DriverCategory::Text;
    let map = registry.entry(category).or_insert_with(HashMap::new);
    
    #[cfg(any(feature = "text", feature = "all"))]
    {
        use crate::drivers::text::*;
        map.insert("text_diff".to_string(), Arc::new(TextDiffDriver));
        map.insert("text_sort".to_string(), Arc::new(TextSortDriver));
        map.insert("text_deduplicate".to_string(), Arc::new(TextDeduplicateDriver));
        map.insert("text_filter".to_string(), Arc::new(TextFilterDriver));
        map.insert("regex_match".to_string(), Arc::new(RegexMatchDriver));
        map.insert("regex_find".to_string(), Arc::new(RegexFindDriver));
        map.insert("regex_replace".to_string(), Arc::new(RegexReplaceDriver));
        map.insert("regex_extract".to_string(), Arc::new(RegexExtractDriver));
    }
}