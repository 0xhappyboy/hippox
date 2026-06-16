/// Skill execution scheduler module
///
/// This module provides the SkillScheduler which orchestrates skill
/// selection and execution based on user input. It uses the skill
/// registry to access built-in atomic skills.
///
/// # Key Responsibilities
/// - Generating skill registry prompts for LLM
/// - Selecting appropriate skills based on user input
/// - Executing skills with parameters
/// - Falling back to general chat when no skill matches
use crate::t;
use futures::future::ok;
use langhub::LLMClient;
use langhub::llms::LLMResult;
use langhub::types::{ChatMessage, LangHubError};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

use hippox_atomic_skills::{
    generate_skill_registry_table_json_str, get_skill_by_name, has_skill, list_skills_names,
};

/// Skill execution scheduler
///
/// Manages the lifecycle of skill execution including:
/// - Skill selection (trigger-based or LLM-driven)
/// - Skill execution with parameter passing
/// - Fallback chat handling
#[derive(Clone)]
pub struct SkillScheduler {
    /// Language model client for LLM interactions
    llm: LLMClient,
}

impl fmt::Debug for SkillScheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SkillScheduler")
            .field("llm", &"<LLMClient>")
            .finish()
    }
}

impl SkillScheduler {
    /// Create a new SkillScheduler instance
    ///
    /// # Arguments
    /// * `llm` - Language model client for making LLM API calls
    pub fn new(llm: LLMClient) -> Self {
        Self { llm }
    }

    /// Generate a comprehensive prompt with all skill metadata from registry
    ///
    /// This prompt includes the complete JSON registry of all available
    /// atomic skills, which the LLM uses to understand what skills are
    /// available and how to call them.
    ///
    /// # Returns
    /// A formatted string containing the skill registry in JSON format
    pub fn get_skills_prompt(&self) -> String {
        let registry_json = generate_skill_registry_table_json_str();
        format!("## Available Skills (JSON Registry)\n{}", registry_json)
    }

    /// Select a skill based on user input
    ///
    /// First attempts trigger-based matching using trigger patterns.
    /// If no trigger matches, asks the LLM to select the most appropriate
    /// skill by name.
    ///
    /// # Arguments
    /// * `user_input` - The user's input text
    ///
    /// # Returns
    /// Some(skill_name) if a skill is selected, None otherwise
    pub async fn select_skill(&self, user_input: &str) -> anyhow::Result<Option<String>> {
        if list_skills_names().is_empty() {
            return Ok(None);
        }
        let skills_prompt = self.get_skills_prompt();
        let select_prompt = format!(
            "{}\n\nAvailable skills:\n{}\n\nUser input: {}\n\nRespond with ONLY the skill name, or 'none' if no skill matches.\n",
            t!("prompt.select_skill_header"),
            skills_prompt,
            user_input
        );
        let result = self.llm.generate(&select_prompt).await?;
        let response = result.text;
        let skill_name = response.trim();
        if skill_name == "none" || skill_name.is_empty() {
            Ok(None)
        } else if has_skill(skill_name) {
            Ok(Some(skill_name.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Execute a skill by name with user input as the parameter
    ///
    /// # Arguments
    /// * `skill_name` - Name of the skill to execute
    /// * `user_input` - User input to pass as the "input" parameter
    /// * `conversation_history` - Previous conversation context (unused in this method)
    ///
    /// # Returns
    /// The skill execution result as a string
    pub async fn execute(
        &self,
        skill_name: &str,
        user_input: &str,
        conversation_history: &str,
    ) -> anyhow::Result<String> {
        println!("{}", t!("skill.executing", skill_name));
        let skill = get_skill_by_name(skill_name)
            .ok_or_else(|| anyhow::anyhow!("Skill not found: {}", skill_name))?;
        let mut parameters = HashMap::new();
        parameters.insert("input".to_string(), Value::String(user_input.to_string()));
        skill.execute(&parameters).await
    }

    /// Execute a skill with explicit parameters
    ///
    /// # Arguments
    /// * `skill_name` - Name of the skill to execute
    /// * `user_input` - Original user input (for logging)
    /// * `parameters` - HashMap of skill-specific parameters
    /// * `conversation_history` - Previous conversation context (unused)
    ///
    /// # Returns
    /// The skill execution result as a string
    pub async fn execute_with_parameters(
        &self,
        skill_name: &str,
        user_input: &str,
        parameters: &HashMap<String, Value>,
        conversation_history: &str,
    ) -> anyhow::Result<String> {
        println!("{}", t!("skill.executing", skill_name));
        let skill = get_skill_by_name(skill_name)
            .ok_or_else(|| anyhow::anyhow!("Skill not found: {}", skill_name))?;
        skill.execute(parameters).await
    }

    /// Execute a skill with chat messages as context
    ///
    /// Extracts the last user message from the chat history and passes
    /// it as the "input" parameter to the skill.
    ///
    /// # Arguments
    /// * `skill_name` - Name of the skill to execute
    /// * `messages` - Vector of chat messages
    ///
    /// # Returns
    /// The skill execution result as a string
    pub async fn execute_with_messages(
        &self,
        skill_name: &str,
        messages: Vec<ChatMessage>,
    ) -> anyhow::Result<String> {
        let skill = get_skill_by_name(skill_name)
            .ok_or_else(|| anyhow::anyhow!("Skill not found: {}", skill_name))?;
        let mut parameters = HashMap::new();
        // Extract content from the last user message
        for msg in messages.iter().rev() {
            if msg.role == "user" {
                parameters.insert("input".to_string(), Value::String(msg.content.clone()));
                break;
            }
        }
        skill.execute(&parameters).await
    }

    /// Fallback chat when no skill matches
    ///
    /// Provides a natural conversation response when the user's request
    /// doesn't match any available skill.
    ///
    /// # Arguments
    /// * `user_input` - The user's input text
    ///
    /// # Returns
    /// A natural language response from the LLM
    pub async fn fallback_chat(&self, user_input: &str) -> anyhow::Result<String> {
        let prompt = format!(
            "{}\n\nYou are a helpful assistant. No specific skill matched the user's request.\n\nUser input: {}\n\nProvide a helpful, natural response to the user.\n",
            t!("prompt.fallback"),
            user_input
        );
        let result = self.llm.generate(&prompt).await?;
        Ok(result.text)
    }

    /// Fallback chat with conversation history
    ///
    /// Similar to fallback_chat but includes previous conversation context
    /// for more coherent responses.
    ///
    /// # Arguments
    /// * `user_input` - The user's input text
    /// * `conversation_history` - Previous conversation context
    ///
    /// # Returns
    /// A natural language response considering the conversation history
    pub async fn fallback_chat_with_history(
        &self,
        user_input: &str,
        conversation_history: &str,
    ) -> anyhow::Result<String> {
        let prompt = format!(
            "{}\n\nYou are a helpful assistant. No specific skill matched the user's request.\n\nPrevious conversation:\n{}\n\nUser input: {}\n\nProvide a helpful, natural response considering the conversation history.\n",
            t!("prompt.fallback"),
            conversation_history,
            user_input
        );
        let result = self.llm.generate(&prompt).await?;
        Ok(result.text)
    }

    /// List all available skills with emoji icons
    ///
    /// # Returns
    /// A formatted string listing all skills with their emoji categories
    pub fn list_skills(&self) -> String {
        let skills = list_skills_names();
        if skills.is_empty() {
            return t!("skill.no_skills_available").to_string();
        }
        let mut result = String::new();
        for name in skills {
            if let Some(skill) = get_skill_by_name(&name) {
                let emoji = skill.category().icon();
                result.push_str(&format!(
                    "   {} - **{}**: {}\n",
                    emoji,
                    name,
                    skill.description()
                ));
            }
        }
        result
    }

    /// Get all available skill names
    ///
    /// # Returns
    /// A vector of skill names
    pub fn get_skill_names(&self) -> Vec<String> {
        list_skills_names()
    }

    /// Check if any skills are available
    ///
    /// # Returns
    /// true if at least one skill is registered, false otherwise
    pub fn has_skills(&self) -> bool {
        !list_skills_names().is_empty()
    }

    /// Get a reference to the LLM client
    ///
    /// # Returns
    /// Reference to the internal LLMClient
    fn get_llm(&self) -> &LLMClient {
        &self.llm
    }

    pub async fn chat_raw(
        &self,
        messages: Vec<ChatMessage>,
    ) -> anyhow::Result<LLMResult, LangHubError> {
        self.llm.chat(messages).await
    }

    /// Generate and return raw LLMResult (with token info, no task tracking)
    pub async fn generate_raw(&self, prompt: &str) -> anyhow::Result<LLMResult, LangHubError> {
        self.llm.generate(prompt).await
    }

    pub async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let messages = vec![ChatMessage::user(prompt)];
        self.chat(messages).await
    }

    /// Chat with LLM (no token tracking)
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> anyhow::Result<String> {
        let result = self.llm.chat(messages).await?;
        Ok(result.text)
    }

    pub async fn generate_with_task(&self, prompt: &str, task_id: &str) -> anyhow::Result<String> {
        let result = self.llm.generate(prompt).await?;
        if let Some(usage) = result.extract_usage() {
            if let Some(updater) = crate::tasks::get_state_updater(task_id).await {
                updater
                    .add_token_usage_global(
                        usage.prompt_tokens as u64,
                        usage.completion_tokens as u64,
                    )
                    .await;
            }
        }
        Ok(result.text)
    }

    /// Chat with LLM with token tracking for a specific task
    pub async fn chat_with_task(
        &self,
        messages: Vec<ChatMessage>,
        task_id: &str,
    ) -> anyhow::Result<String> {
        let result = self.llm.chat(messages).await?;
        if let Some(usage) = result.extract_usage() {
            if let Some(updater) = crate::tasks::get_state_updater(task_id).await {
                updater
                    .add_token_usage_global(
                        usage.prompt_tokens as u64,
                        usage.completion_tokens as u64,
                    )
                    .await;
            }
        }
        Ok(result.text)
    }
}

#[cfg(test)]
mod skill_scheduler_test {
    use super::*;
    use langhub::LLMClient;
    use langhub::types::ModelProvider;

    /// Create a test scheduler with OpenAI provider
    fn create_test_scheduler() -> SkillScheduler {
        let llm = LLMClient::new_with_key(
            ModelProvider::OpenAI,
            Some("test-api-key".to_string()),
            None,
        )
        .unwrap();
        SkillScheduler::new(llm)
    }

    #[test]
    fn test_list_skills() {
        let scheduler = create_test_scheduler();
        let list = scheduler.list_skills();
        // Registry should have at least helloworld skill
        assert!(list.contains("helloworld"));
    }

    #[test]
    fn test_get_skill_names() {
        let scheduler = create_test_scheduler();
        let names = scheduler.get_skill_names();
        assert!(names.contains(&"helloworld".to_string()));
        assert!(names.contains(&"calculator".to_string()));
        assert!(names.contains(&"file_read".to_string()));
    }

    #[test]
    fn test_has_skills() {
        let scheduler = create_test_scheduler();
        assert!(scheduler.has_skills());
    }

    #[test]
    fn test_get_skills_prompt() {
        let scheduler = create_test_scheduler();
        let prompt = scheduler.get_skills_prompt();
        assert!(prompt.contains("Available Skills"));
        assert!(prompt.contains("helloworld"));
        assert!(prompt.contains("calculator"));
    }

    #[tokio::test]
    async fn test_select_skill_with_trigger() {
        let scheduler = create_test_scheduler();
        // This test requires actual LLM call, so we skip it in normal test runs
        // Use integration tests for actual LLM calls
        let result = scheduler.select_skill("calculate 2+3").await;
        assert!(result.is_ok());
    }
}
