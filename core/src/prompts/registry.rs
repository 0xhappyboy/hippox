//! Registry generation for skills and instances

use crate::get_config;
use hippox_atomic_skills::{get_skills_by_categories, skill_registry::{self, get_registry, get_skill, list_skills}};
use serde_json::{Value, json};

/// Generate skills registry (atomic skills metadata)
pub fn generate_skills_registry() -> String {
    let skills = skill_registry::list_skills();
    let registry: Vec<serde_json::Value> = skills
        .iter()
        .filter_map(|name| {
            skill_registry::get_skill(name).map(|skill| {
                serde_json::json!({
                    "name": name,
                    "description": skill.description(),
                    "category": skill.category(),
                    "parameters": skill.parameters(),
                    "example_call": skill.example_call(),
                    "example_output": skill.example_output(),
                })
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
    let skills = get_skills_by_categories(categories);
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
    let skills = list_skills();
    let registry: Vec<serde_json::Value> = skills
        .iter()
        .filter_map(|name| {
            get_skill(name).map(|skill| {
                serde_json::json!({
                    "name": name,
                    "desc": skill.description(),
                    "category": skill.category(),
                })
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
    let skills = get_skills_by_categories(categories);
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
    let registry = get_registry();
    let mut categories: std::collections::HashSet<String> = std::collections::HashSet::new();

    for skill in registry.values() {
        categories.insert(skill.category().to_string());
    }

    let mut result: Vec<String> = categories.into_iter().collect();
    result.sort();
    result
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
