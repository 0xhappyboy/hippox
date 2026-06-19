//! Registry generation for drivers and instances

use crate::get_config;
use hippox_drivers::{
    get_all_categorys, get_all_drivers, get_registry, get_driver_by_name, get_drivers_by_category,
    get_drivers_by_category_list, list_drivers_names,
};
use serde_json::{Value, json};

/// Generate drivers registry (atomic drivers metadata)
pub fn generate_drivers_registry() -> String {
    let drivers = get_all_drivers();
    let registry: Vec<serde_json::Value> = drivers
        .iter()
        .map(|driver| {
            serde_json::json!({
                "name": driver.name(),
                "description": driver.description(),
                "category": driver.category().name(),
                "parameters": driver.parameters(),
                "example_call": driver.example_call(),
                "example_output": driver.example_output(),
            })
        })
        .collect();
    serde_json::to_string_pretty(&registry).unwrap_or_else(|_| "[]".to_string())
}

/// Generate filtered drivers registry by categories
pub fn generate_drivers_registry_by_categories(categories: &[String]) -> String {
    if categories.is_empty() {
        return "[]".to_string();
    }
    let drivers = get_drivers_by_category_list(categories);
    let registry: Vec<serde_json::Value> = drivers
        .iter()
        .map(|driver| {
            serde_json::json!({
                "name": driver.name(),
                "description": driver.description(),
                "category": driver.category(),
                "parameters": driver.parameters(),
                "example_call": driver.example_call(),
                "example_output": driver.example_output(),
            })
        })
        .collect();
    serde_json::to_string_pretty(&registry).unwrap_or_else(|_| "[]".to_string())
}

/// Generate minimal drivers registry (only name, desc, category) - for ultra-low token usage
pub fn generate_minimal_drivers_registry() -> String {
    let drivers = get_all_drivers();
    let registry: Vec<serde_json::Value> = drivers
        .iter()
        .map(|driver| {
            serde_json::json!({
                "name": driver.name(),
                "desc": driver.description(),
                "category": driver.category().name(),
            })
        })
        .collect();
    serde_json::to_string_pretty(&registry).unwrap_or_else(|_| "[]".to_string())
}

/// Generate filtered minimal drivers registry by categories
pub fn generate_minimal_drivers_registry_by_categories(categories: &[String]) -> String {
    if categories.is_empty() {
        return "[]".to_string();
    }
    let drivers = get_drivers_by_category_list(categories);
    let registry: Vec<serde_json::Value> = drivers
        .iter()
        .map(|driver| {
            serde_json::json!({
                "name": driver.name(),
                "desc": driver.description(),
                "category": driver.category(),
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
