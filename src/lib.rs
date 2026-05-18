mod config;
mod core;
mod envs;
mod executors;
mod global;
mod i18n;
mod skill_loader;
mod skill_scheduler;
mod types;

pub use config::{GLOBAL_CONFIG, HippoxConfig, get_config, init_config_from_env};
pub use core::Hippox;
pub use langhub::types::ModelProvider;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn create_test_skill_md(dir: &tempfile::TempDir, skill_name: &str, description: &str) {
        let skill_dir = dir.path().join(skill_name);
        std::fs::create_dir_all(&skill_dir).unwrap();
        let skill_md = skill_dir.join("SKILL.md");
        let content = format!(
            r#"---
name: {}
description: {}
version: 1.0.0
author: Test Author
---

# {} Skill

This is a test workflow for {}.

## Instructions
Process the request and return a result.
"#,
            skill_name, description, skill_name, description
        );
        std::fs::write(skill_md, content).unwrap();
    }

    #[tokio::test]
    async fn test_hippox_new() {
        let temp_dir = tempdir().unwrap();
        let hippox = Hippox::new(
            temp_dir.path().to_str().unwrap(),
            ModelProvider::OpenAI,
            "en",
        )
        .await;
        assert!(hippox.is_ok());
    }

    #[tokio::test]
    async fn test_list_atomic_skills() {
        let temp_dir = tempdir().unwrap();
        let hippox = Hippox::new(
            temp_dir.path().to_str().unwrap(),
            ModelProvider::OpenAI,
            "en",
        )
        .await
        .unwrap();
        let skills = hippox.list_atomic_skills();
        assert!(skills.contains("calculator") || skills.contains("helloworld"));
    }

    #[tokio::test]
    async fn test_list_skill_md_files() {
        let temp_dir = tempdir().unwrap();
        create_test_skill_md(&temp_dir, "test-skill", "A test skill");
        let hippox = Hippox::new(
            temp_dir.path().to_str().unwrap(),
            ModelProvider::OpenAI,
            "en",
        )
        .await
        .unwrap();
        let list = hippox.list_skill_md_files();
        assert!(list.contains("test-skill"));
    }

    #[tokio::test]
    async fn test_clear_conversation() {
        let temp_dir = tempdir().unwrap();
        let hippox = Hippox::new(
            temp_dir.path().to_str().unwrap(),
            ModelProvider::OpenAI,
            "en",
        )
        .await
        .unwrap();
        hippox.clear_conversation("test-session");
        hippox.clear_all_conversations();
    }

    #[test]
    fn test_extract_json() {
        let text = r#"Some text {"action": "calculator", "parameters": {"input": "2+2"}}"#;
        let json = Hippox::extract_json(text);
        assert!(json.contains("calculator"));
        let text = "```json\n{\"action\": \"test\"}\n```";
        let json = Hippox::extract_json(text);
        assert_eq!(json, "{\"action\": \"test\"}");
    }

    #[test]
    fn test_get_atomic_skill_names() {
        let temp_dir = tempdir().unwrap();
        let hippox = tokio::runtime::Runtime::new().unwrap().block_on(async {
            Hippox::new(
                temp_dir.path().to_str().unwrap(),
                ModelProvider::OpenAI,
                "en",
            )
            .await
            .unwrap()
        });
        let names = hippox.get_atomic_skill_names();
        assert!(!names.is_empty());
        assert!(names.contains(&"calculator".to_string()));
    }

    #[test]
    fn test_has_atomic_skills() {
        let temp_dir = tempdir().unwrap();
        let hippox = tokio::runtime::Runtime::new().unwrap().block_on(async {
            Hippox::new(
                temp_dir.path().to_str().unwrap(),
                ModelProvider::OpenAI,
                "en",
            )
            .await
            .unwrap()
        });
        assert!(hippox.has_atomic_skills());
    }

    #[test]
    fn test_skills_directory() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().to_str().unwrap();
        let hippox = tokio::runtime::Runtime::new().unwrap().block_on(async {
            Hippox::new(path, ModelProvider::OpenAI, "en")
                .await
                .unwrap()
        });
        assert_eq!(hippox.skills_directory().to_str().unwrap(), path);
    }
}
