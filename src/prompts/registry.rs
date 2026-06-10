//! Registry generation for skills and instances

use crate::get_config;
use serde_json::{Value, json};

/// Generate skills registry (atomic skills metadata)
pub fn generate_skills_registry() -> String {
    let skills = crate::executors::registry::list_skills();
    let registry: Vec<serde_json::Value> = skills
        .iter()
        .filter_map(|name| {
            crate::executors::registry::get_skill(name).map(|skill| {
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
