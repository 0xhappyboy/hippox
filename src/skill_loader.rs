/// SKILL.md file loader module
///
/// This module provides functionality to load and parse SKILL.md files
/// from the skills directory. SKILL.md files define the metadata and
/// instructions for community-contributed skills.
use crate::t;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Default SKILL.md filename
const SKILL_FILE_NAME: &str = "SKILL.md";

/// Minimum directory depth for skill scanning
const SKILL_FILE_SCAN_MIN_DEPTH: usize = 1;

/// Maximum directory depth for skill scanning
const SKILL_FILE_SCAN_MAX_DEPTH: usize = 1;

/// Trigger patterns for automatic skill activation
///
/// When user input matches any of these patterns, the skill can be
/// automatically selected without LLM decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTrigger {
    /// List of trigger patterns (keywords or phrases)
    pub patterns: Vec<String>,
    /// Whether pattern matching is case-sensitive
    #[serde(default)]
    pub case_sensitive: bool,
}

/// Metadata extracted from SKILL.md frontmatter
///
/// Contains additional information about the skill such as author,
/// version, emoji, and platform requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFrontmatterMetadata {
    /// Skill author name or GitHub username
    pub author: Option<String>,
    /// Skill version (semantic versioning recommended)
    pub version: Option<String>,
    /// Emoji icon for visual representation
    pub emoji: Option<String>,
    /// Supported operating systems (linux, macos, windows)
    pub os: Option<Vec<String>>,
    /// External dependencies or requirements
    pub requires: Option<HashMap<String, serde_json::Value>>,
    /// Additional arbitrary metadata fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Complete SKILL.md file structure
///
/// Represents a parsed SKILL.md file containing skill metadata
/// and natural language instructions for the AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFile {
    /// Unique skill identifier (used as action in SkillCall)
    pub name: String,
    /// Brief description of what the skill does
    pub description: String,
    /// Semantic version of the skill
    pub version: Option<String>,
    /// License type (MIT, Apache, etc.)
    pub license: Option<String>,
    /// Original author of the skill
    pub author: Option<String>,
    /// Compatibility constraints
    pub compatibility: Option<String>,
    /// Trigger patterns for automatic activation
    pub triggers: Option<SkillTrigger>,
    /// List of tools this skill is allowed to use
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// Other skills this skill depends on
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Frontmatter metadata
    pub metadata: Option<SkillFrontmatterMetadata>,
    /// Parameter definitions for the skill
    #[serde(default)]
    pub parameters: Vec<crate::executors::types::SkillParameter>,
    /// Natural language instructions for AI execution
    pub instructions: String,
    /// Original file path
    pub path: PathBuf,
}

/// Frontmatter intermediate parsing structure
///
/// Used during YAML frontmatter deserialization before converting
/// to the final SkillFile structure.
#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    /// Unique skill identifier
    name: String,
    /// Brief description of what the skill does
    description: String,
    /// Semantic version of the skill
    version: Option<String>,
    /// License type
    license: Option<String>,
    /// Original author
    author: Option<String>,
    /// Compatibility constraints
    compatibility: Option<String>,
    /// Trigger patterns for automatic activation
    triggers: Option<SkillTrigger>,
    /// List of tools this skill is allowed to use
    #[serde(default)]
    allowed_tools: Vec<String>,
    /// Other skills this skill depends on
    #[serde(default)]
    dependencies: Vec<String>,
    /// Frontmatter metadata
    metadata: Option<SkillFrontmatterMetadata>,
    /// Parameter definitions
    #[serde(default)]
    parameters: Vec<crate::executors::types::SkillParameter>,
    /// Additional arbitrary fields for forward compatibility
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

/// SKILL.md file loader
///
/// Provides static methods to load and parse SKILL.md files
/// from the filesystem. This is used for community-contributed
/// skills that are written in Markdown format.
pub struct SkillLoader;

impl SkillLoader {
    /// Load all SKILL.md files from a directory
    ///
    /// Scans the specified directory for subdirectories containing
    /// SKILL.md files. Each subdirectory at depth 1 is expected to
    /// contain a SKILL.md file.
    ///
    /// # Arguments
    /// * `skills_dir` - Path to the skills directory
    ///
    /// # Returns
    /// A vector of parsed SkillFile structures
    ///
    /// # Errors
    /// Returns error if the directory does not exist
    pub fn load_all(skills_dir: &str) -> anyhow::Result<Vec<SkillFile>> {
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

    /// Load a single SKILL.md file by skill name
    ///
    /// # Arguments
    /// * `skills_dir` - Path to the skills directory
    /// * `name` - Name of the skill (also used as subdirectory name)
    ///
    /// # Returns
    /// Some(SkillFile) if found, None otherwise
    pub fn load_by_name(skills_dir: &str, name: &str) -> anyhow::Result<Option<SkillFile>> {
        let skill_path = Path::new(skills_dir).join(name).join(SKILL_FILE_NAME);
        if skill_path.exists() {
            Ok(Some(Self::parse_skill_file(&skill_path)?))
        } else {
            Ok(None)
        }
    }

    /// Parse a SKILL.md file from the given path
    ///
    /// # Arguments
    /// * `path` - Path to the SKILL.md file
    ///
    /// # Returns
    /// A parsed SkillFile structure
    fn parse_skill_file(path: &Path) -> anyhow::Result<SkillFile> {
        let content = fs::read_to_string(path)?;
        let (frontmatter, instructions) = Self::parse_frontmatter(&content)?;
        Ok(SkillFile {
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
    ///
    /// Extracts YAML frontmatter (between --- markers) and the
    /// remaining markdown content as instructions.
    ///
    /// # Arguments
    /// * `content` - Raw markdown content
    ///
    /// # Returns
    /// A tuple containing (frontmatter, instructions)
    ///
    /// # Errors
    /// Returns error if frontmatter is malformed or missing
    fn parse_frontmatter(content: &str) -> anyhow::Result<(SkillFrontmatter, String)> {
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            anyhow::bail!(t!("error.invalid_skill_format"));
        }
        let frontmatter: SkillFrontmatter = serde_yaml::from_str(parts[1])?;
        let instructions = parts[2].trim().to_string();
        Ok((frontmatter, instructions))
    }

    /// Load a SKILL.md file from a specific path
    ///
    /// # Arguments
    /// * `path` - Direct path to the SKILL.md file
    ///            Example: "./skills/web-search/SKILL.md"
    ///
    /// # Returns
    /// Some(SkillFile) if found and valid, None otherwise
    pub fn load_from_path(path: &str) -> anyhow::Result<Option<SkillFile>> {
        let skill_path = Path::new(path);
        if skill_path.exists() && skill_path.is_file() {
            Ok(Some(Self::parse_skill_file(skill_path)?))
        } else {
            Ok(None)
        }
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
}
