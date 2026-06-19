//! Intent parser types
//!
//! This module defines the data structures for parsing user intent.

use serde::{Deserialize, Serialize};

/// Result from intent parser LLM call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentParseResult {
    /// Clean user intent without any formatting instructions
    pub clean_intent: String,
    /// driver categories needed to fulfill the request
    pub skill_categories: Vec<String>,
}

impl IntentParseResult {
    /// Create an empty result (for fallback when parsing fails)
    pub fn empty() -> Self {
        Self {
            clean_intent: String::new(),
            skill_categories: Vec::new(),
        }
    }

    /// Create a fallback result using original input
    pub fn fallback(original_input: &str) -> Self {
        Self {
            clean_intent: original_input.to_string(),
            skill_categories: Vec::new(),
        }
    }

    /// Check if the result is valid (has non-empty clean_intent)
    pub fn is_valid(&self) -> bool {
        !self.clean_intent.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_parse_result_empty() {
        let result = IntentParseResult::empty();
        assert!(result.clean_intent.is_empty());
        assert!(result.skill_categories.is_empty());
        assert!(!result.is_valid());
    }
}
