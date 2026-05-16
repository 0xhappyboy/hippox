use crate::skill_loader::Skill;
use crate::t;
use langhub::LLMClient;
use langhub::types::ChatMessage;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone)]
pub struct SkillScheduler {
    skills: Vec<Skill>,
    llm: LLMClient,
    skill_cache: HashMap<String, Skill>,
}

impl SkillScheduler {
    pub fn new(skills: Vec<Skill>, llm: LLMClient) -> Self {
        let mut skill_cache = HashMap::new();
        for skill in &skills {
            skill_cache.insert(skill.name.clone(), skill.clone());
        }
        Self {
            skills,
            llm,
            skill_cache,
        }
    }

    /// Generate a comprehensive prompt with all skill metadata
    pub fn get_skills_prompt(&self) -> String {
        if self.skills.is_empty() {
            return t!("skill.no_skills_available").to_string();
        }
        let mut result = String::new();
        for skill in &self.skills {
            result.push_str(&format!("## {}\n", skill.name));
            result.push_str(&format!("**Description:** {}\n", skill.description));
            if let Some(version) = &skill.version {
                result.push_str(&format!("**Version:** {}\n", version));
            }
            if let Some(author) = &skill.author {
                result.push_str(&format!("**Author:** {}\n", author));
            }
            if let Some(triggers) = &skill.triggers {
                result.push_str(&format!("**Triggers:** {}\n", triggers.patterns.join(", ")));
            }
            if !skill.parameters.is_empty() {
                result.push_str("**Parameters:**\n");
                for param in &skill.parameters {
                    let required = if param.required {
                        "(required)"
                    } else {
                        "(optional)"
                    };
                    result.push_str(&format!(
                        "  - `{}` ({}): {} {}\n",
                        param.name, param.param_type, param.description, required
                    ));
                    if let Some(default) = &param.default {
                        result.push_str(&format!("    Default: {}\n", default));
                    }
                }
            }
            if !skill.allowed_tools.is_empty() {
                result.push_str(&format!(
                    "**Allowed Tools:** {}\n",
                    skill.allowed_tools.join(", ")
                ));
            }
            if let Some(metadata) = &skill.metadata {
                if let Some(emoji) = &metadata.emoji {
                    result.push_str(&format!("**Emoji:** {}\n", emoji));
                }
            }
            result.push('\n');
        }
        result
    }

    /// Generate full skill context for selection
    fn get_skill_context(&self, skill: &Skill) -> String {
        let mut context = String::new();
        context.push_str(&format!("# Skill: {}\n\n", skill.name));
        context.push_str(&format!("## Description\n{}\n\n", skill.description));
        if let Some(version) = &skill.version {
            context.push_str(&format!("**Version:** {}\n", version));
        }
        if !skill.parameters.is_empty() {
            context.push_str("## Parameters\n");
            for param in &skill.parameters {
                let required = if param.required {
                    "Required"
                } else {
                    "Optional"
                };
                context.push_str(&format!(
                    "- **{}** ({}): {} - {}\n",
                    param.name, param.param_type, param.description, required
                ));
                if let Some(default) = &param.default {
                    context.push_str(&format!("  Default: {}\n", default));
                }
            }
            context.push('\n');
        }
        if !skill.allowed_tools.is_empty() {
            context.push_str(&format!(
                "## Allowed Tools\n{}\n\n",
                skill.allowed_tools.join(", ")
            ));
        }
        if !skill.dependencies.is_empty() {
            context.push_str(&format!(
                "## Dependencies\n{}\n\n",
                skill.dependencies.join(", ")
            ));
        }
        if let Some(triggers) = &skill.triggers {
            context.push_str(&format!(
                "## Triggers\nPatterns: {}\nCase sensitive: {}\n\n",
                triggers.patterns.join(", "),
                triggers.case_sensitive
            ));
        }
        context.push_str("## Instructions\n");
        context.push_str(&skill.instructions);
        context.push_str("\n\n");
        context
    }

    pub async fn select_skill(&self, user_input: &str) -> anyhow::Result<Option<&Skill>> {
        if self.skills.is_empty() {
            return Ok(None);
        }
        for skill in &self.skills {
            if let Some(triggers) = &skill.triggers {
                let patterns = &triggers.patterns;
                let user_lower = if triggers.case_sensitive {
                    user_input.to_string()
                } else {
                    user_input.to_lowercase()
                };
                for pattern in patterns {
                    let pattern_match = if triggers.case_sensitive {
                        pattern.clone()
                    } else {
                        pattern.to_lowercase()
                    };
                    if user_lower.contains(&pattern_match) {
                        return Ok(Some(skill));
                    }
                }
            }
        }
        let skills_prompt = self.get_skills_prompt();
        let select_prompt = format!(
            "{}\n\n{}\n\n{}\n\n{}\n\n{}",
            t!("prompt.select_skill_header"),
            "Available skills:",
            skills_prompt,
            t!("prompt.select_skill_footer", user_input),
            "Respond with ONLY the skill name, or 'none' if no skill matches."
        );
        let response = self.llm.generate(&select_prompt).await?;
        let skill_name = response.trim();
        if skill_name == "none" || skill_name.is_empty() {
            Ok(None)
        } else {
            Ok(self.skills.iter().find(|s| s.name == skill_name))
        }
    }

    pub async fn execute(
        &self,
        skill: &Skill,
        user_input: &str,
        conversation_history: &str,
    ) -> anyhow::Result<String> {
        println!("{}", t!("skill.executing", &skill.name));
        println!("{}", t!("skill.description", &skill.description));
        if let Some(version) = &skill.version {
            println!("Version: {}", version);
        }
        if let Some(emoji) = skill.metadata.as_ref().and_then(|m| m.emoji.as_ref()) {
            println!("{} Executing skill", emoji);
        }
        println!("{}", t!("skill.instructions"));
        println!("{}", t!("skill.user_input", user_input));
        let skill_context = self.get_skill_context(skill);
        let execution_prompt = format!(
            "{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}",
            t!("prompt.execute_skill_header"),
            "## Skill Context",
            skill_context,
            t!("prompt.previous_conversation", conversation_history),
            t!("prompt.user_input", user_input),
            "Follow the skill instructions exactly. Use the allowed tools if specified. Return the final response to the user."
        );
        let response = self.llm.generate(&execution_prompt).await?;
        Ok(response)
    }

    pub async fn execute_with_parameters(
        &self,
        skill: &Skill,
        user_input: &str,
        parameters: &HashMap<String, Value>,
        conversation_history: &str,
    ) -> anyhow::Result<String> {
        let mut param_context = String::new();
        if !parameters.is_empty() {
            param_context.push_str("## Parameters\n");
            for (key, value) in parameters {
                param_context.push_str(&format!("- {}: {}\n", key, value));
            }
            param_context.push('\n');
        }
        let skill_context = self.get_skill_context(skill);
        let execution_prompt = format!(
            "{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}",
            t!("prompt.execute_skill_header"),
            "## Skill Context",
            skill_context,
            param_context,
            t!("prompt.previous_conversation", conversation_history),
            t!("prompt.user_input", user_input),
            "Execute this skill with the provided parameters. Follow the instructions carefully."
        );
        let response = self.llm.generate(&execution_prompt).await?;
        Ok(response)
    }

    pub async fn execute_with_messages(
        &self,
        skill: &Skill,
        messages: Vec<ChatMessage>,
    ) -> anyhow::Result<String> {
        let skill_context = self.get_skill_context(skill);
        let system_prompt = format!(
            "{}\n\n{}\n\n{}\n\n{}",
            t!("prompt.system_prompt_header"),
            "## Skill Context",
            skill_context,
            t!("prompt.system_prompt_footer")
        );
        let mut full_messages = vec![ChatMessage::system(&system_prompt)];
        full_messages.extend(messages);
        let response = self.llm.chat(full_messages).await?;
        Ok(response)
    }

    pub async fn fallback_chat(&self, user_input: &str) -> anyhow::Result<String> {
        let prompt = format!(
            "{}\n\n{}\n\n{}\n\n{}",
            t!("prompt.fallback"),
            "You are a helpful assistant. No specific skill matched the user's request.",
            t!("prompt.user_input", user_input),
            "Provide a helpful, natural response to the user."
        );
        let response = self.llm.generate(&prompt).await?;
        Ok(response)
    }

    pub async fn fallback_chat_with_history(
        &self,
        user_input: &str,
        conversation_history: &str,
    ) -> anyhow::Result<String> {
        let prompt = format!(
            "{}\n\n{}\n\n{}\n\n{}\n\n{}",
            t!("prompt.fallback"),
            "You are a helpful assistant. No specific skill matched the user's request.",
            t!("prompt.previous_conversation", conversation_history),
            t!("prompt.user_input", user_input),
            "Provide a helpful, natural response considering the conversation history."
        );
        let response = self.llm.generate(&prompt).await?;
        Ok(response)
    }

    pub fn list_skills(&self) -> String {
        if self.skills.is_empty() {
            return t!("skill.no_skills_available").to_string();
        }
        let mut result = String::new();
        for skill in &self.skills {
            let emoji = skill
                .metadata
                .as_ref()
                .and_then(|m| m.emoji.as_ref())
                .map(|e| format!("{} ", e))
                .unwrap_or_default();
            result.push_str(&format!(
                "   {}- **{}**: {}\n",
                emoji, skill.name, skill.description
            ));
            if let Some(version) = &skill.version {
                result.push_str(&format!("      Version: {}\n", version));
            }
            if let Some(triggers) = &skill.triggers {
                result.push_str(&format!(
                    "      Triggers: {}\n",
                    triggers.patterns.join(", ")
                ));
            }
        }
        result
    }

    pub fn get_skill(&self, name: &str) -> Option<&Skill> {
        self.skill_cache.get(name)
    }

    pub fn get_all_skills(&self) -> &Vec<Skill> {
        &self.skills
    }

    pub fn has_skills(&self) -> bool {
        !self.skills.is_empty()
    }

    pub fn reload_skills(&mut self, skills_dir: &str) -> anyhow::Result<()> {
        let new_skills = crate::skill_loader::SkillLoader::load_all(skills_dir)?;
        self.skills = new_skills;
        self.skill_cache.clear();
        for skill in &self.skills {
            self.skill_cache.insert(skill.name.clone(), skill.clone());
        }
        Ok(())
    }

    pub fn get_llm(&self) -> &LLMClient {
        &self.llm
    }
}

#[cfg(test)]
mod skill_scheduler_test {
    use crate::skill_loader::SkillTrigger;

    use super::*;
    use langhub::LLMClient;
    use langhub::types::ModelProvider;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_skill() -> Skill {
        Skill {
            name: "test-skill".to_string(),
            description: "A test skill for unit testing".to_string(),
            version: Some("1.0.0".to_string()),
            license: None,
            author: Some("Test Author".to_string()),
            compatibility: None,
            triggers: Some(SkillTrigger {
                patterns: vec!["test".to_string(), "demo".to_string()],
                case_sensitive: false,
            }),
            allowed_tools: vec!["http".to_string()],
            dependencies: vec![],
            metadata: None,
            parameters: vec![],
            instructions: "Do something useful".to_string(),
            path: std::path::PathBuf::new(),
        }
    }

    fn create_test_scheduler() -> SkillScheduler {
        let llm = LLMClient::new(ModelProvider::OpenAI).unwrap();
        let skills = vec![create_test_skill()];
        SkillScheduler::new(skills, llm)
    }

    #[test]
    fn test_list_skills() {
        let scheduler = create_test_scheduler();
        let list = scheduler.list_skills();
        assert!(list.contains("test-skill"));
        assert!(list.contains("Version: 1.0.0"));
        assert!(list.contains("Triggers: test, demo"));
    }

    #[test]
    fn test_get_skill() {
        let scheduler = create_test_scheduler();
        let skill = scheduler.get_skill("test-skill");
        assert!(skill.is_some());
        assert_eq!(skill.unwrap().name, "test-skill");
    }

    #[test]
    fn test_get_nonexistent_skill() {
        let scheduler = create_test_scheduler();
        let skill = scheduler.get_skill("nonexistent");
        assert!(skill.is_none());
    }

    #[test]
    fn test_has_skills() {
        let scheduler = create_test_scheduler();
        assert!(scheduler.has_skills());

        let llm = LLMClient::new(ModelProvider::OpenAI).unwrap();
        let empty_scheduler = SkillScheduler::new(vec![], llm);
        assert!(!empty_scheduler.has_skills());
    }

    #[test]
    fn test_get_skills_prompt() {
        let scheduler = create_test_scheduler();
        let prompt = scheduler.get_skills_prompt();
        assert!(prompt.contains("test-skill"));
        assert!(prompt.contains("A test skill for unit testing"));
        assert!(prompt.contains("Triggers: test, demo"));
    }

    #[test]
    fn test_skill_context_generation() {
        let skill = create_test_skill();
        let scheduler = create_test_scheduler();
        let context = scheduler.get_skill_context(&skill);
        assert!(context.contains("# Skill: test-skill"));
        assert!(context.contains("## Instructions"));
        assert!(context.contains("Do something useful"));
        assert!(context.contains("## Triggers"));
        assert!(context.contains("test, demo"));
    }

    #[test]
    fn test_reload_skills() {
        let temp_dir = tempdir().unwrap();
        let skills_dir = temp_dir.path();

        let skill_subdir = skills_dir.join("reload-skill");
        fs::create_dir(&skill_subdir).unwrap();

        let skill_content = r#"---
name: reload-skill
description: A skill that can be reloaded
version: 1.0.0
---

# Instructions
Do something.
"#;

        fs::write(skill_subdir.join("SKILL.md"), skill_content).unwrap();

        let llm = LLMClient::new(ModelProvider::OpenAI).unwrap();
        let mut scheduler = SkillScheduler::new(vec![], llm);
        assert!(!scheduler.has_skills());

        scheduler
            .reload_skills(skills_dir.to_str().unwrap())
            .unwrap();
        assert!(scheduler.has_skills());
        assert!(scheduler.get_skill("reload-skill").is_some());
    }
}
