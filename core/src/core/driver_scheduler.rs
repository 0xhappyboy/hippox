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
use crate::{HippoxStringResult, t};
use futures::future::ok;
use hippox_drivers::{
    DriverCallback, DriverContext, DriverError, DriverResult, generate_driver_registry_table_json_str, get_driver_by_name, has_driver,
    list_drivers_names,
};
use langhub::LLMClient;
use langhub::llms::LLMResult;
use langhub::types::{ChatMessage, LangHubError};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use tracing::{debug, info, warn};
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
        return f.debug_struct("DriverScheduler").field("llm", &"<LLMClient>").finish();
    }
}
impl DriverScheduler {
    /// Create a new DriverScheduler instance
    ///
    /// # Arguments
    /// * `llm` - Language model client for making LLM API calls
    pub fn new(llm: LLMClient) -> Self {
        debug!("Creating new DriverScheduler instance");
        return Self { llm };
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
        debug!("Generating drivers prompt");
        let registry_json = generate_driver_registry_table_json_str();
        let prompt = format!("## Available Drivers (JSON Registry)\n{}", registry_json);
        info!("Drivers prompt generated, length: {}", prompt.len());
        return prompt;
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
        debug!("Selecting driver for input: {}", user_input);
        let driver_names = list_drivers_names();
        if driver_names.is_empty() {
            info!("No drivers available in registry");
            return Ok(None);
        }
        info!("Found {} drivers in registry", driver_names.len());
        let drivers_prompt = self.get_drivers_prompt();
        let select_prompt = format!(
            "{}\n\nAvailable drivers:\n{}\n\nUser input: {}\n\nRespond with ONLY the driver name, or 'none' if no driver matches.\n",
            t!("prompt.select_driver_header"),
            drivers_prompt,
            user_input
        );
        debug!("Sending driver selection prompt to LLM");
        let result = self.llm.generate(&select_prompt).await?;
        let response = result.text;
        let driver_name = response.trim();
        debug!("LLM response for driver selection: '{}'", driver_name);
        if driver_name == "none" || driver_name.is_empty() {
            info!("No driver selected by LLM");
            return Ok(None);
        } else if has_driver(driver_name) {
            info!("Driver selected: {}", driver_name);
            return Ok(Some(driver_name.to_string()));
        } else {
            warn!("Driver '{}' not found in registry", driver_name);
            return Ok(None);
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
        debug!("Executing driver: {}, input: {}, history_len: {}", driver_name, user_input, conversation_history.len());
        println!("{}", t!("driver.executing", driver_name));
        let driver = get_driver_by_name(driver_name).ok_or_else(|| anyhow::anyhow!("Driver not found: {}", driver_name))?;
        let mut parameters = HashMap::new();
        parameters.insert("input".to_string(), Value::String(user_input.to_string()));
        info!("Driver {} execution started with parameters: {:?}", driver_name, parameters.keys());
        // Convert DriverResult to anyhow::Result
        let result = driver.execute(&parameters, driver_callback, driver_context).await.map_err(|e| anyhow::anyhow!("{}", e))?;
        info!("Driver {} execution completed successfully", driver_name);
        return Ok(result);
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
        debug!("Executing driver with parameters: {}, input: {}, history_len: {}", driver_name, user_input, conversation_history.len());
        println!("{}", t!("driver.executing", driver_name));
        let driver = get_driver_by_name(driver_name).ok_or_else(|| anyhow::anyhow!("Driver not found: {}", driver_name))?;
        info!("Driver {} execution with {} parameters", driver_name, parameters.len());
        // Convert DriverResult to anyhow::Result
        let result = driver.execute(parameters, driver_callback, driver_context).await.map_err(|e| anyhow::anyhow!("{}", e))?;
        info!("Driver {} execution with parameters completed successfully", driver_name);
        return Ok(result);
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
    /// The driver execution result as a HippoxStringResult
    pub async fn execute_with_messages(
        &self,
        driver_name: &str,
        messages: Vec<ChatMessage>,
        driver_callback: Option<&dyn DriverCallback>,
        driver_context: Option<&DriverContext>,
    ) -> HippoxStringResult {
        debug!("Executing driver with messages: {}, messages_count: {}", driver_name, messages.len());
        // Get the driver from registry
        let driver = match get_driver_by_name(driver_name) {
            Some(d) => {
                info!("Driver '{}' found in registry", driver_name);
                d
            }
            None => {
                let err_msg = format!("Driver not found: {}", driver_name);
                warn!("{}", err_msg);
                return HippoxStringResult::system_error(err_msg);
            }
        };
        let mut parameters = HashMap::new();
        // Extract content from the last user message
        let mut input_found = false;
        for msg in messages.iter().rev() {
            if msg.role == "user" {
                parameters.insert("input".to_string(), Value::String(msg.content.clone()));
                input_found = true;
                debug!("Found user message with content length: {}", msg.content.len());
                break;
            }
        }
        if !input_found {
            let err_msg = "No user message found in conversation history".to_string();
            warn!("{}", err_msg);
            return HippoxStringResult::system_error(err_msg);
        }
        info!("Driver {} execution with {} parameters from messages", driver_name, parameters.len());
        // Execute the driver
        match driver.execute(&parameters, driver_callback, driver_context).await {
            Ok(result) => {
                info!("Driver {} executed successfully with messages", driver_name);
                return HippoxStringResult::ok(result);
            }
            Err(e) => {
                let err_msg = format!("Driver execution failed: {}", e);
                warn!("{}", err_msg);
                return HippoxStringResult::system_error(err_msg);
            }
        }
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
        debug!("Falling back to chat for input: {}", user_input);
        let prompt = format!(
            "{}\n\nYou are a helpful assistant. No specific driver matched the user's request.\n\nUser input: {}\n\nProvide a helpful, natural response to the user.\n",
            t!("prompt.fallback"),
            user_input
        );
        info!("Fallback chat prompt length: {}", prompt.len());
        let result = self.llm.generate(&prompt).await?;
        info!("Fallback chat response generated, length: {}", result.text.len());
        return Ok(result.text);
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
    pub async fn fallback_chat_with_history(&self, user_input: &str, conversation_history: &str) -> anyhow::Result<String> {
        debug!("Falling back to chat with history, input: {}, history_len: {}", user_input, conversation_history.len());
        let prompt = format!(
            "{}\n\nYou are a helpful assistant. No specific driver matched the user's request.\n\nPrevious conversation:\n{}\n\nUser input: {}\n\nProvide a helpful, natural response considering the conversation history.\n",
            t!("prompt.fallback"),
            conversation_history,
            user_input
        );
        info!("Fallback chat with history prompt length: {}", prompt.len());
        let result = self.llm.generate(&prompt).await?;
        info!("Fallback chat with history response generated, length: {}", result.text.len());
        return Ok(result.text);
    }
    /// List all available drivers with emoji icons
    ///
    /// # Returns
    /// A formatted string listing all drivers with their emoji categories
    pub fn list_drivers(&self) -> String {
        debug!("Listing all available drivers");
        let drivers = list_drivers_names();
        if drivers.is_empty() {
            info!("No drivers available");
            return t!("driver.no_drivers_available").to_string();
        }
        info!("Found {} drivers to list", drivers.len());
        let mut result = String::new();
        for name in drivers {
            if let Some(driver) = get_driver_by_name(&name) {
                let category = driver.category();
                let emoji = category.icon();
                result.push_str(&format!("   {} - **{}**: {}\n", emoji, name, driver.description()));
            }
        }
        debug!("Drivers list generated, length: {}", result.len());
        return result;
    }
    /// Get all available driver names
    ///
    /// # Returns
    /// A vector of driver names
    pub fn get_driver_names(&self) -> Vec<String> {
        debug!("Getting all driver names");
        let names = list_drivers_names();
        info!("Retrieved {} driver names", names.len());
        return names;
    }
    /// Check if any drivers are available
    ///
    /// # Returns
    /// true if at least one driver is registered, false otherwise
    pub fn has_drivers(&self) -> bool {
        let has = !list_drivers_names().is_empty();
        debug!("Has drivers: {}", has);
        return has;
    }
    /// Get a reference to the LLM client
    ///
    /// # Returns
    /// Reference to the internal LLMClient
    fn get_llm(&self) -> &LLMClient {
        return &self.llm;
    }
    /// Chat with LLM and return raw LLMResult
    pub async fn chat_raw(&self, messages: Vec<ChatMessage>) -> anyhow::Result<LLMResult, LangHubError> {
        debug!("Chat raw with {} messages", messages.len());
        let result = self.llm.chat(messages).await?;
        info!("Chat raw completed, response length: {}", result.text.len());
        return Ok(result);
    }
    /// Generate and return raw LLMResult (with token info, no task tracking)
    pub async fn generate_raw(&self, prompt: &str) -> anyhow::Result<LLMResult, LangHubError> {
        debug!("Generate raw with prompt length: {}", prompt.len());
        let result = self.llm.generate(prompt).await?;
        info!("Generate raw completed, response length: {}", result.text.len());
        return Ok(result);
    }
    /// Generate a response from LLM
    pub async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        debug!("Generate with prompt length: {}", prompt.len());
        let messages = vec![ChatMessage::user(prompt)];
        let result = self.chat(messages).await?;
        info!("Generate completed, response length: {}", result.len());
        return Ok(result);
    }
    /// Chat with LLM (no token tracking)
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> anyhow::Result<String> {
        debug!("Chat with {} messages", messages.len());
        let result = self.llm.chat(messages).await?;
        info!("Chat completed, response length: {}", result.text.len());
        return Ok(result.text);
    }
    /// Generate with task tracking for token usage
    pub async fn generate_with_task(&self, prompt: &str, task_id: &str) -> anyhow::Result<String> {
        debug!("Generate with task: {}, prompt length: {}", task_id, prompt.len());
        let result = self.llm.generate(prompt).await?;
        if let Some(usage) = result.extract_usage() {
            if let Some(updater) = crate::tasks::get_state_updater(task_id).await {
                updater.add_token_usage_global(usage.prompt_tokens as u64, usage.completion_tokens as u64).await;
                info!("Token usage updated for task {}: prompt={}, completion={}", task_id, usage.prompt_tokens, usage.completion_tokens);
            }
        }
        info!("Generate with task completed, response length: {}", result.text.len());
        return Ok(result.text);
    }
    /// Chat with LLM with token tracking for a specific task
    pub async fn chat_with_task(&self, messages: Vec<ChatMessage>, task_id: &str) -> anyhow::Result<String> {
        debug!("Chat with task: {}, messages_count: {}", task_id, messages.len());
        let result = self.llm.chat(messages).await?;
        if let Some(usage) = result.extract_usage() {
            if let Some(updater) = crate::tasks::get_state_updater(task_id).await {
                updater.add_token_usage_global(usage.prompt_tokens as u64, usage.completion_tokens as u64).await;
                info!("Token usage updated for task {}: prompt={}, completion={}", task_id, usage.prompt_tokens, usage.completion_tokens);
            }
        }
        info!("Chat with task completed, response length: {}", result.text.len());
        return Ok(result.text);
    }
}
#[cfg(test)]
mod driver_scheduler_test {
    use super::*;
    use langhub::LLMClient;
    use langhub::types::ModelProvider;
    /// Create a test scheduler with OpenAI provider
    fn create_test_scheduler() -> DriverScheduler {
        let llm = LLMClient::new_with_key(ModelProvider::OpenAI, Some("test-api-key".to_string()), None).unwrap();
        return DriverScheduler::new(llm);
    }
    #[test]
    fn test_list_drivers() {
        let scheduler = create_test_scheduler();
        let list = scheduler.list_drivers();
        println!("Drivers List: {:?}", list);
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
