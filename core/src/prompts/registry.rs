//! Registry generation for skills and instances

use crate::get_config;
use hippox_atomic_skills::{
    get_all_categorys, get_all_skills, get_registry, get_skill_by_name, get_skills_by_category,
    get_skills_by_category_list, list_skills_names,
};
use serde_json::{Value, json};

/// Generate skills registry (atomic skills metadata)
pub fn generate_skills_registry() -> String {
    let skills = get_all_skills();
    let registry: Vec<serde_json::Value> = skills
        .iter()
        .map(|skill| {
            serde_json::json!({
                "name": skill.name(),
                "description": skill.description(),
                "category": skill.category().name(),
                "parameters": skill.parameters(),
                "example_call": skill.example_call(),
                "example_output": skill.example_output(),
            })
        })
        .collect();
    serde_json::to_string_pretty(&registry).unwrap_or_else(|_| "[]".to_string())
}

/// Generate filtered skills registry by categories
pub fn generate_skills_registry_by_categories(categories: &[String]) -> String {
    if categories.is_empty() {
        return "[]".to_string();
    }
    let skills = get_skills_by_category_list(categories);
    let registry: Vec<serde_json::Value> = skills
        .iter()
        .map(|skill| {
            serde_json::json!({
                "name": skill.name(),
                "description": skill.description(),
                "category": skill.category(),
                "parameters": skill.parameters(),
                "example_call": skill.example_call(),
                "example_output": skill.example_output(),
            })
        })
        .collect();
    serde_json::to_string_pretty(&registry).unwrap_or_else(|_| "[]".to_string())
}

/// Generate minimal skills registry (only name, desc, category) - for ultra-low token usage
pub fn generate_minimal_skills_registry() -> String {
    let skills = get_all_skills();
    let registry: Vec<serde_json::Value> = skills
        .iter()
        .map(|skill| {
            serde_json::json!({
                "name": skill.name(),
                "desc": skill.description(),
                "category": skill.category().name(),
            })
        })
        .collect();
    serde_json::to_string_pretty(&registry).unwrap_or_else(|_| "[]".to_string())
}

/// Generate filtered minimal skills registry by categories
pub fn generate_minimal_skills_registry_by_categories(categories: &[String]) -> String {
    if categories.is_empty() {
        return "[]".to_string();
    }
    let skills = get_skills_by_category_list(categories);
    let registry: Vec<serde_json::Value> = skills
        .iter()
        .map(|skill| {
            serde_json::json!({
                "name": skill.name(),
                "desc": skill.description(),
                "category": skill.category(),
            })
        })
        .collect();
    serde_json::to_string_pretty(&registry).unwrap_or_else(|_| "[]".to_string())
}

pub fn get_all_categories() -> Vec<String> {
    get_all_categorys()
}

#[cfg(test)]
mod tests {
    use crate::prompts::build_intent_parser_prompt;

    use super::*;

    #[test]
    fn test_get_all_categories() {
        let prompts = build_intent_parser_prompt("test");
        println!("prompts {:?}", prompts);
    }
}
