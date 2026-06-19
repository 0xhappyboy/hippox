/// Driver execution scheduler module
///
/// This module provides the DriverScheduler which orchestrates driver
/// selection and execution based on user input. It uses the driver
/// registry to access built-in atomic drivers.
///
/// # Key Responsibilities
/// - Generating driver registry prompts for LLM
/// - Selecting appropriate drivers based on user input
/// - Executing drivers with parameters
/// - Falling back to general chat when no driver matches
use crate::t;
use futures::future::ok;
use langhub::LLMClient;
use langhub::llms::LLMResult;
use langhub::types::{ChatMessage, LangHubError};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

use hippox_drivers::{
    DriverCallback, DriverContext, generate_driver_registry_table_json_str, get_driver_by_name,
    has_driver, list_drivers_names,
};

/// Driver execution scheduler
///
/// Manages the lifecycle of driver execution including:
/// - Driver selection (trigger-based or LLM-driven)
/// - Driver execution with parameter passing
/// - Fallback chat handling
#[derive(Clone)]
pub struct DriverScheduler {
    /// Language model client for LLM interactions
    llm: LLMClient,
}

impl fmt::Debug for DriverScheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DriverScheduler")
            .field("llm", &"<LLMClient>")
            .finish()
    }
}

impl DriverScheduler {
    /// Create a new DriverScheduler instance
    ///
    /// # Arguments
    /// * `llm` - Language model client for making LLM API calls
    pub fn new(llm: LLMClient) -> Self {
        Self { llm }
    }

    /// Generate a comprehensive prompt with all driver metadata from registry
    ///
    /// This prompt includes the complete JSON registry of all available
    /// atomic drivers, which the LLM uses to understand what drivers are
    /// available and how to call them.
    ///
    /// # Returns
    /// A formatted string containing the driver registry in JSON format
    pub fn get_drivers_prompt(&self) -> String {
        let registry_json = generate_driver_registry_table_json_str();
        format!("## Available Drivers (JSON Registry)\n{}", registry_json)
    }

    /// Select a driver based on user input
    ///
    /// First attempts trigger-based matching using trigger patterns.
    /// If no trigger matches, asks the LLM to select the most appropriate
    /// driver by name.
    ///
    /// # Arguments
    /// * `user_input` - The user's input text
    ///
    /// # Returns
    /// Some(driver_name) if a driver is selected, None otherwise
    pub async fn select_driver(&self, user_input: &str) -> anyhow::Result<Option<String>> {
        if list_drivers_names().is_empty() {
            return Ok(None);
        }
        let drivers_prompt = self.get_drivers_prompt();
        let select_prompt = format!(
            "{}\n\nAvailable drivers:\n{}\n\nUser input: {}\n\nRespond with ONLY the driver name, or 'none' if no driver matches.\n",
            t!("prompt.select_driver_header"),
            drivers_prompt,
            user_input
        );
        let result = self.llm.generate(&select_prompt).await?;
        let response = result.text;
        let driver_name = response.trim();
        if driver_name == "none" || driver_name.is_empty() {
            Ok(None)
        } else if has_driver(driver_name) {
            Ok(Some(driver_name.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Execute a driver by name with user input as the parameter
    ///
    /// # Arguments
    /// * `driver_name` - Name of the driver to execute
    /// * `user_input` - User input to pass as the "input" parameter
    /// * `conversation_history` - Previous conversation context (unused in this method)
    ///
    /// # Returns
    /// The driver execution result as a string
    pub async fn execute(
        &self,
        driver_name: &str,
        user_input: &str,
        conversation_history: &str,
        driver_callback: Option<&dyn DriverCallback>,
        driver_context: Option<&DriverContext>,
    ) -> anyhow::Result<String> {
        println!("{}", t!("driver.executing", driver_name));
        let driver = get_driver_by_name(driver_name)
            .ok_or_else(|| anyhow::anyhow!("Driver not found: {}", driver_name))?;
        let mut parameters = HashMap::new();
        parameters.insert("input".to_string(), Value::String(user_input.to_string()));
        driver
            .execute(&parameters, driver_callback, driver_context)
            .await
    }

    /// Execute a driver with explicit parameters
    ///
    /// # Arguments
    /// * `driver_name` - Name of the driver to execute
    /// * `user_input` - Original user input (for logging)
    /// * `parameters` - HashMap of driver-specific parameters
    /// * `conversation_history` - Previous conversation context (unused)
    ///
    /// # Returns
    /// The driver execution result as a string
    pub async fn execute_with_parameters(
        &self,
        driver_name: &str,
        user_input: &str,
        parameters: &HashMap<String, Value>,
        conversation_history: &str,
        driver_callback: Option<&dyn DriverCallback>,
        driver_context: Option<&DriverContext>,
    ) -> anyhow::Result<String> {
        println!("{}", t!("driver.executing", driver_name));
        let driver = get_driver_by_name(driver_name)
            .ok_or_else(|| anyhow::anyhow!("Driver not found: {}", driver_name))?;
        driver
            .execute(parameters, driver_callback, driver_context)
            .await
    }

    /// Execute a driver with chat messages as context
    ///
    /// Extracts the last user message from the chat history and passes
    /// it as the "input" parameter to the driver.
    ///
    /// # Arguments
    /// * `driver_name` - Name of the driver to execute
    /// * `messages` - Vector of chat messages
    ///
    /// # Returns
    /// The driver execution result as a string
    pub async fn execute_with_messages(
        &self,
        driver_name: &str,
        messages: Vec<ChatMessage>,
        driver_callback: Option<&dyn DriverCallback>,
        driver_context: Option<&DriverContext>,
    ) -> anyhow::Result<String> {
        let driver = get_driver_by_name(driver_name)
            .ok_or_else(|| anyhow::anyhow!("Driver not found: {}", driver_name))?;
        let mut parameters = HashMap::new();
        // Extract content from the last user message
        for msg in messages.iter().rev() {
            if msg.role == "user" {
                parameters.insert("input".to_string(), Value::String(msg.content.clone()));
                break;
            }
        }
        driver
            .execute(&parameters, driver_callback, driver_context)
            .await
    }

    /// Fallback chat when no driver matches
    ///
    /// Provides a natural conversation response when the user's request
    /// doesn't match any available driver.
    ///
    /// # Arguments
    /// * `user_input` - The user's input text
    ///
    /// # Returns
    /// A natural language response from the LLM
    pub async fn fallback_chat(&self, user_input: &str) -> anyhow::Result<String> {
        let prompt = format!(
            "{}\n\nYou are a helpful assistant. No specific driver matched the user's request.\n\nUser input: {}\n\nProvide a helpful, natural response to the user.\n",
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
            "{}\n\nYou are a helpful assistant. No specific driver matched the user's request.\n\nPrevious conversation:\n{}\n\nUser input: {}\n\nProvide a helpful, natural response considering the conversation history.\n",
            t!("prompt.fallback"),
            conversation_history,
            user_input
        );
        let result = self.llm.generate(&prompt).await?;
        Ok(result.text)
    }

    /// List all available drivers with emoji icons
    ///
    /// # Returns
    /// A formatted string listing all drivers with their emoji categories
    pub fn list_drivers(&self) -> String {
        let drivers = list_drivers_names();
        if drivers.is_empty() {
            return t!("driver.no_drivers_available").to_string();
        }
        let mut result = String::new();
        for name in drivers {
            if let Some(driver) = get_driver_by_name(&name) {
                let emoji = driver.category().icon();
                result.push_str(&format!(
                    "   {} - **{}**: {}\n",
                    emoji,
                    name,
                    driver.description()
                ));
            }
        }
        result
    }

    /// Get all available driver names
    ///
    /// # Returns
    /// A vector of driver names
    pub fn get_driver_names(&self) -> Vec<String> {
        list_drivers_names()
    }

    /// Check if any drivers are available
    ///
    /// # Returns
    /// true if at least one driver is registered, false otherwise
    pub fn has_drivers(&self) -> bool {
        !list_drivers_names().is_empty()
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
mod driver_scheduler_test {
    use super::*;
    use langhub::LLMClient;
    use langhub::types::ModelProvider;

    /// Create a test scheduler with OpenAI provider
    fn create_test_scheduler() -> DriverScheduler {
        let llm = LLMClient::new_with_key(
            ModelProvider::OpenAI,
            Some("test-api-key".to_string()),
            None,
        )
        .unwrap();
        DriverScheduler::new(llm)
    }

    #[test]
    fn test_list_drivers() {
        let scheduler = create_test_scheduler();
        let list = scheduler.list_drivers();
        // Registry should have at least helloworld driver
        assert!(list.contains("helloworld"));
    }

    #[test]
    fn test_get_driver_names() {
        let scheduler = create_test_scheduler();
        let names = scheduler.get_driver_names();
        assert!(names.contains(&"helloworld".to_string()));
        assert!(names.contains(&"calculator".to_string()));
        assert!(names.contains(&"file_read".to_string()));
    }

    #[test]
    fn test_has_drivers() {
        let scheduler = create_test_scheduler();
        assert!(scheduler.has_drivers());
    }

    #[test]
    fn test_get_drivers_prompt() {
        let scheduler = create_test_scheduler();
        let prompt = scheduler.get_drivers_prompt();
        assert!(prompt.contains("Available Drivers"));
        assert!(prompt.contains("helloworld"));
        assert!(prompt.contains("calculator"));
    }

    #[tokio::test]
    async fn test_select_driver_with_trigger() {
        let scheduler = create_test_scheduler();
        // This test requires actual LLM call, so we skip it in normal test runs
        // Use integration tests for actual LLM calls
        let result = scheduler.select_driver("calculate 2+3").await;
        assert!(result.is_ok());
    }
}
