//! Registry generation for skills and instances

use crate::{
    get_config,
    skill_registry::{get_registry, get_skill, get_skills_by_categories, list_skills},
};
use serde_json::{Value, json};

/// Generate skills registry (atomic skills metadata)
pub fn generate_skills_registry() -> String {
    let skills = crate::executors::skill_registry::list_skills();
    let registry: Vec<serde_json::Value> = skills
        .iter()
        .filter_map(|name| {
            crate::executors::skill_registry::get_skill(name).map(|skill| {
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

/// Generate instances registry (configured database/service instances)
pub fn generate_instances_registry() -> String {
    let config = get_config();
    let mut instances = serde_json::Map::new();
    // PostgreSQL instances
    let pg_instances: Vec<serde_json::Value> = config
        .postgresql_instances
        .values()
        .map(|inst| {
            json!({
                "id": inst.id,
                "name": inst.name,
                "description": inst.description,
                "type": "postgresql"
            })
        })
        .collect();
    if !pg_instances.is_empty() {
        instances.insert("postgresql".to_string(), json!(pg_instances));
    }
    // MySQL instances
    let mysql_instances: Vec<serde_json::Value> = config
        .mysql_instances
        .values()
        .map(|inst| {
            json!({
                "id": inst.id,
                "name": inst.name,
                "description": inst.description,
                "type": "mysql"
            })
        })
        .collect();
    if !mysql_instances.is_empty() {
        instances.insert("mysql".to_string(), json!(mysql_instances));
    }
    // Redis instances
    let redis_instances: Vec<serde_json::Value> = config
        .redis_instances
        .values()
        .map(|inst| {
            json!({
                "id": inst.id,
                "name": inst.name,
                "description": inst.description,
                "type": "redis"
            })
        })
        .collect();
    if !redis_instances.is_empty() {
        instances.insert("redis".to_string(), json!(redis_instances));
    }
    // SQLite instances
    let sqlite_instances: Vec<serde_json::Value> = config
        .sqlite_instances
        .values()
        .map(|inst| {
            json!({
                "id": inst.id,
                "name": inst.name,
                "description": inst.description,
                "type": "sqlite"
            })
        })
        .collect();
    if !sqlite_instances.is_empty() {
        instances.insert("sqlite".to_string(), json!(sqlite_instances));
    }
    // Docker instances
    let docker_instances: Vec<serde_json::Value> = config
        .docker_instances
        .values()
        .map(|inst| {
            json!({
                "id": inst.id,
                "name": inst.name,
                "description": inst.description,
                "type": "docker"
            })
        })
        .collect();
    if !docker_instances.is_empty() {
        instances.insert("docker".to_string(), json!(docker_instances));
    }
    // Kubernetes instances
    let k8s_instances: Vec<serde_json::Value> = config
        .k8s_instances
        .values()
        .map(|inst| {
            json!({
                "id": inst.id,
                "name": inst.name,
                "description": inst.description,
                "type": "kubernetes",
                "namespace": inst.namespace
            })
        })
        .collect();
    if !k8s_instances.is_empty() {
        instances.insert("kubernetes".to_string(), json!(k8s_instances));
    }
    serde_json::to_string_pretty(&instances).unwrap_or_else(|_| "{}".to_string())
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
