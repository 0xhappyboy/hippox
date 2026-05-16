use crate::t;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// skill file dir name
const SKILL_FILE_DIR: &str = "skills";
/// skill file name
const SKILL_FILE_NAME: &str = "SKILL.md";
/// skill file scan min depth
const SKILL_FILE_SCAN_MIN_DEPTH: usize = 1;
/// skill file scan max depth
const SKILL_FILE_SCAN_MAX_DEPTH: usize = 1;

/// Skill parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
}

/// Skill trigger patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTrigger {
    pub patterns: Vec<String>,
    #[serde(default)]
    pub case_sensitive: bool,
}

/// Metadata fields supporting arbitrary JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub author: Option<String>,
    pub version: Option<String>,
    pub emoji: Option<String>,
    pub os: Option<Vec<String>>,
    pub requires: Option<HashMap<String, serde_json::Value>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Complete Skill structure matching Lobster spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    // Required fields
    pub name: String,
    pub description: String,
    // Optional fields
    pub version: Option<String>,
    pub license: Option<String>,
    pub author: Option<String>,
    pub compatibility: Option<String>,
    // Trigger conditions
    pub triggers: Option<SkillTrigger>,
    // Tool permissions
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    // Dependencies on other skills
    #[serde(default)]
    pub dependencies: Vec<String>,
    // Metadata
    pub metadata: Option<SkillMetadata>,
    // Parameter definitions
    #[serde(default)]
    pub parameters: Vec<SkillParameter>,
    // Main instructions content (supports full Markdown structure)
    pub instructions: String,
    // File path
    pub path: PathBuf,
}

/// Frontmatter intermediate parsing structure
#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    // Required
    name: String,
    description: String,
    // Optional base fields
    version: Option<String>,
    license: Option<String>,
    author: Option<String>,
    compatibility: Option<String>,
    // Triggers
    triggers: Option<SkillTrigger>,
    // Tool permissions
    #[serde(default)]
    allowed_tools: Vec<String>,
    // Dependencies
    #[serde(default)]
    dependencies: Vec<String>,
    // Metadata
    metadata: Option<SkillMetadata>,
    // Parameter definitions
    #[serde(default)]
    parameters: Vec<SkillParameter>,
    // Allow extra unknown fields for forward compatibility
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

pub struct SkillLoader;

impl SkillLoader {
    /// Load all skills from directory
    pub fn load_all(skills_dir: &str) -> anyhow::Result<Vec<Skill>> {
        let mut skills = Vec::new();
        let skills_path = Path::new(skills_dir);
        if !skills_path.exists() {
            anyhow::bail!(t!("error.config_not_found", skills_dir));
        }
        for entry in WalkDir::new(skills_path)
            .min_depth(SKILL_FILE_SCAN_MIN_DEPTH)
            .max_depth(SKILL_FILE_SCAN_MAX_DEPTH)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_dir() {
                let skill_file = path.join(SKILL_FILE_NAME);
                if skill_file.exists() {
                    if let Ok(skill) = Self::parse_skill_file(&skill_file) {
                        skills.push(skill);
                    }
                }
            }
        }
        Ok(skills)
    }

    /// Load a single skill by name
    pub fn load_by_name(skills_dir: &str, name: &str) -> anyhow::Result<Option<Skill>> {
        let skill_path = Path::new(skills_dir).join(name).join("SKILL.md");
        if skill_path.exists() {
            Ok(Some(Self::parse_skill_file(&skill_path)?))
        } else {
            Ok(None)
        }
    }

    /// Parse a SKILL.md file
    fn parse_skill_file(path: &Path) -> anyhow::Result<Skill> {
        let content = fs::read_to_string(path)?;
        let (frontmatter, instructions) = Self::parse_frontmatter(&content)?;
        Ok(Skill {
            name: frontmatter.name,
            description: frontmatter.description,
            version: frontmatter.version,
            license: frontmatter.license,
            author: frontmatter.author,
            compatibility: frontmatter.compatibility,
            triggers: frontmatter.triggers,
            allowed_tools: frontmatter.allowed_tools,
            dependencies: frontmatter.dependencies,
            metadata: frontmatter.metadata,
            parameters: frontmatter.parameters,
            instructions,
            path: path.to_path_buf(),
        })
    }

    /// Parse frontmatter from markdown content
    fn parse_frontmatter(content: &str) -> anyhow::Result<(SkillFrontmatter, String)> {
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            anyhow::bail!(t!("error.invalid_skill_format"));
        }
        let frontmatter: SkillFrontmatter = serde_yaml::from_str(parts[1])?;
        let instructions = parts[2].trim().to_string();
        Ok((frontmatter, instructions))
    }

    /// Scan and export all skills as a JSON registry Table (hot reload)
    pub fn get_skills_registry_table_json(skills_dir: &str) -> anyhow::Result<serde_json::Value> {
        let skills = Self::load_all(skills_dir)?;
        let registry_skills: Vec<serde_json::Value> = skills
            .iter()
            .map(|skill| {
                serde_json::json!({
                    "name": skill.name,
                    "description": skill.description,
                    "emoji": skill.metadata.as_ref().and_then(|m| m.emoji.clone()),
                    "version": skill.version,
                    "parameters": skill.parameters,
                    "triggers": skill.triggers.as_ref().map(|t| &t.patterns),
                    "allowed_tools": skill.allowed_tools,
                })
            })
            .collect();
        Ok(serde_json::json!({
            "version": "1.0",
            "total_skills": registry_skills.len(),
            "skills": registry_skills,
        }))
    }

    /// Scan and export all skills as a JSON String registry Table (hot reload)
    pub fn get_skills_registry_table_json_str(skills_dir: &str) -> anyhow::Result<String> {
        let registry = Self::get_skills_registry_table_json(skills_dir)?;
        Ok(serde_json::to_string_pretty(&registry)?)
    }
}

#[cfg(test)]
mod skill_loader_test {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_complete_skill() {
        let content = r#"---
name: web-search
description: Search the web using a search engine
version: 1.0.0
author: Lobster Team
license: MIT
compatibility: ">=1.0.0"
triggers:
  patterns:
    - "search for"
    - "find online"
    - "google"
  case_sensitive: false
allowed_tools:
  - http
  - json
dependencies:
  - url-parser
metadata:
  author: lobster
  version: 1.0.0
  emoji: 🔍
  os:
    - linux
    - macos
  requires:
    api_key: true
parameters:
  - name: query
    type: string
    description: The search query
    required: true
  - name: limit
    type: integer
    description: Maximum number of results
    required: false
    default: 10
---

# Web Search Skill

This skill performs web searches.

## Steps

1. Parse the user's search query
2. Call the search API with the query
3. Return formatted results

## Examples

User: "search for Rust programming"
Response: Here are the top 10 results...

## Error Handling

If the API fails, return a friendly error message.
"#;
        let (frontmatter, instructions) = SkillLoader::parse_frontmatter(content).unwrap();
        assert_eq!(frontmatter.name, "web-search");
        assert_eq!(
            frontmatter.description,
            "Search the web using a search engine"
        );
        assert_eq!(frontmatter.version, Some("1.0.0".to_string()));
        assert_eq!(frontmatter.author, Some("Lobster Team".to_string()));
        assert!(instructions.contains("Web Search Skill"));
        assert!(instructions.contains("## Steps"));
        assert!(instructions.contains("## Examples"));
        let triggers = frontmatter.triggers.unwrap();
        assert_eq!(triggers.patterns.len(), 3);
        assert!(!triggers.case_sensitive);
        assert_eq!(frontmatter.allowed_tools, vec!["http", "json"]);
        assert_eq!(frontmatter.dependencies, vec!["url-parser"]);
        assert_eq!(frontmatter.parameters.len(), 2);
        assert_eq!(frontmatter.parameters[0].name, "query");
        assert_eq!(frontmatter.parameters[0].param_type, "string");
        assert!(frontmatter.parameters[0].required);
    }

    #[test]
    fn test_parse_minimal_skill() {
        let content = r#"---
name: simple-skill
description: A simple test skill
---

# Simple Instructions

Just do something simple.
"#;

        let (frontmatter, instructions) = SkillLoader::parse_frontmatter(content).unwrap();
        assert_eq!(frontmatter.name, "simple-skill");
        assert_eq!(frontmatter.description, "A simple test skill");
        assert!(instructions.contains("Simple Instructions"));
        assert!(frontmatter.version.is_none());
        assert!(frontmatter.allowed_tools.is_empty());
        assert!(frontmatter.parameters.is_empty());
    }

    #[test]
    fn test_parse_skill_with_triggers() {
        let content = r#"---
name: calculator
description: Perform mathematical calculations
triggers:
  patterns:
    - "calculate"
    - "what is"
    - "compute"
    - "math"
  case_sensitive: false
allowed_tools:
  - math
---

# Calculator Skill

Perform calculations based on user input.
"#;

        let (frontmatter, _) = SkillLoader::parse_frontmatter(content).unwrap();

        let triggers = frontmatter.triggers.unwrap();
        assert_eq!(
            triggers.patterns,
            vec!["calculate", "what is", "compute", "math"]
        );
        assert!(!triggers.case_sensitive);
    }

    #[test]
    fn test_parse_skill_with_parameters() {
        let content = r#"---
name: file-processor
description: Process files in various formats
parameters:
  - name: file_path
    type: string
    description: Path to the file
    required: true
  - name: format
    type: string
    description: Output format
    required: false
    default: json
  - name: verbose
    type: boolean
    description: Enable verbose output
    required: false
    default: false
---

# File Processor

Process the file according to parameters.
"#;

        let (frontmatter, _) = SkillLoader::parse_frontmatter(content).unwrap();

        assert_eq!(frontmatter.parameters.len(), 3);
        assert_eq!(frontmatter.parameters[0].name, "file_path");
        assert_eq!(frontmatter.parameters[0].param_type, "string");
        assert!(frontmatter.parameters[0].required);
        assert_eq!(
            frontmatter.parameters[1].default,
            Some(serde_json::Value::String("json".to_string()))
        );
        assert_eq!(frontmatter.parameters[2].param_type, "boolean");
    }

    #[test]
    fn test_parse_skill_with_metadata() {
        let content = r#"---
name: data-analyzer
description: Analyze data and generate reports
metadata:
  author: data-team
  version: 2.1.0
  emoji: 📊
  os:
    - windows
    - linux
  requires:
    python: ">=3.8"
    memory: "4GB"
---

# Data Analyzer

Analyze the provided data.
"#;

        let (frontmatter, _) = SkillLoader::parse_frontmatter(content).unwrap();

        let metadata = frontmatter.metadata.unwrap();
        assert_eq!(metadata.author, Some("data-team".to_string()));
        assert_eq!(metadata.version, Some("2.1.0".to_string()));
        assert_eq!(metadata.emoji, Some("📊".to_string()));
        assert_eq!(
            metadata.os,
            Some(vec!["windows".to_string(), "linux".to_string()])
        );

        let requires = metadata.requires.unwrap();
        assert_eq!(
            requires.get("python"),
            Some(&serde_json::Value::String(">=3.8".to_string()))
        );
        assert_eq!(
            requires.get("memory"),
            Some(&serde_json::Value::String("4GB".to_string()))
        );
    }

    #[test]
    fn test_parse_invalid_skill() {
        let content = "No frontmatter here at all";
        let result = SkillLoader::parse_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_skills_registry_json_functions() {
        let skills_dir = format!("./{}", SKILL_FILE_DIR).to_string();
        let registry_value = SkillLoader::get_skills_registry_table_json(&skills_dir).unwrap();
        println!("{:?}", registry_value);
        println!("=== Registry Value ===");
        println!("{}", serde_json::to_string_pretty(&registry_value).unwrap());
        assert_eq!(registry_value["version"], "1.0");
        println!("Total skills loaded: {}", registry_value["total_skills"]);
        for (i, skill) in registry_value["skills"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            println!("  {}. {} - {}", i + 1, skill["name"], skill["description"]);
        }
        let json_string = SkillLoader::get_skills_registry_table_json_str(&skills_dir).unwrap();
        println!("\n=== JSON String (first 500 chars) ===");
        println!("{}", &json_string[..json_string.len().min(500)]);
        let parsed: serde_json::Value = serde_json::from_str(&json_string).unwrap();
        assert_eq!(parsed["version"], "1.0");
        assert_eq!(parsed["total_skills"], registry_value["total_skills"]);
        assert!(parsed["total_skills"].as_u64().unwrap() > 0);
        println!("\nloaded {} skills", parsed["total_skills"]);
    }

    #[test]
    fn test_export_skills_registry_json_functions2() {
        let skills_dir = format!("./{}", SKILL_FILE_DIR).to_string();
        match SkillLoader::load_all(&skills_dir) {
            Ok(skills) => {
                println!("Loaded {} skills", skills.len());
                for skill in &skills {
                    println!("  - {}: {}", skill.name, skill.description);
                }
            }
            Err(e) => {
                println!("Error loading skills: {}", e);
            }
        }
        let registry_value = SkillLoader::get_skills_registry_table_json(&skills_dir).unwrap();
        println!("{}", serde_json::to_string_pretty(&registry_value).unwrap());
    }
}
