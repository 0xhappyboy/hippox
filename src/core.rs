use crate::config::{
    init_config_from_env, init_config_from_json_file, init_config_from_params_json_str,
    init_config_from_toml_file,
};
use crate::executors::Executor;
use crate::memory::ConversationMemory;
use crate::skill_loader::SkillLoader;
use crate::skill_scheduler::SkillScheduler;
use crate::{HippoxConfig, i18n};
use crate::{get_config, t};
use langhub::LLMClient;
use langhub::types::ModelProvider;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

/// System prompt template for natural language processing
const SYSTEM_PROMPT_TEMPLATE: &str = r#"You are an AI assistant that can execute atomic skills/tools.

## Available Atomic Skills (JSON Registry)
{}

## Response Format

You can respond in one of three ways:

### 1. Execute a single skill
{"action": "skill_name", "parameters": {"param1": "value1"}}

### 2. Execute multiple skills in sequence (no dependencies)
{
  "mode": "batch",
  "steps": [
    {"action": "skill1", "parameters": {}},
    {"action": "skill2", "parameters": {}}
  ]
}

### 3. Finish and return final answer
{"action": "done", "message": "Your final answer here"}

## Rules

- If the task requires conditional logic (e.g., "if rain then send email"), use mode "single" and execute one skill at a time
- After each skill execution, you will receive the result and can decide the next step
- Use "batch" mode only when skills have no dependencies on each other's results
- Use "done" when you have completed the task or no skill is needed

## Previous Execution Results (if any)
"#;

/// Step result for multi-step execution
#[derive(Debug, Clone)]
pub struct StepResult {
    pub skill: String,
    pub parameters: HashMap<String, Value>,
    pub output: String,
}

impl StepResult {
    pub fn to_string(&self) -> String {
        format!(
            "Executed skill '{}' with parameters {:?}\nResult: {}",
            self.skill, self.parameters, self.output
        )
    }
}

pub enum ConfigInitMethod {
    Env,
    TomlFile(String),
    JsonFile(String),
    ParamsJsonStr(String),
}

/// Core engine for Hippox
///
/// This is the main entry point for the Hippox engine. It handles:
/// - Natural language processing with atomic skill registry
/// - SKILL.md file execution for complex workflows
/// - Managing conversation history for natural language interactions
#[derive(Clone)]
pub struct Hippox {
    scheduler: SkillScheduler,
    executor: Executor,
    memory: ConversationMemory,
    skills_dir: PathBuf,
}

impl Hippox {
    /// Create a new Hippox core instance
    ///
    /// # Arguments
    /// * `skills_dir` - Path to the directory containing SKILL.md subdirectories
    /// * `provider` - LLM provider to use (OpenAI, etc.)
    /// * `api_key` - API key for the LLM provider
    /// * `extra_keys` - Optional additional keys for providers that need them (e.g., secret_key, endpoint)
    /// * `lang` - Language for i18n
    pub async fn new(
        skills_dir: &str,
        provider: ModelProvider,
        api_key: Option<String>,
        extra_keys: Option<HashMap<String, String>>,
        config_method: ConfigInitMethod,
    ) -> anyhow::Result<Self> {
        info!(
            "Initializing Hippox core with skills directory: {}",
            skills_dir
        );
        // init config
        match config_method {
            ConfigInitMethod::Env => init_config_from_env(),
            ConfigInitMethod::TomlFile(path) => init_config_from_toml_file(&path)?,
            ConfigInitMethod::JsonFile(path) => init_config_from_json_file(&path)?,
            ConfigInitMethod::ParamsJsonStr(json) => init_config_from_params_json_str(&json)?,
        }
        // set i18n
        let config = get_config();
        i18n::set_language(&config.lang);
        // init llm
        let llm = LLMClient::new_with_key(provider, api_key, extra_keys)?;
        // init llm scheduler
        let scheduler = SkillScheduler::new(llm);
        let executor = Executor::new();
        Ok(Self {
            scheduler,
            executor,
            memory: ConversationMemory::new(),
            skills_dir: PathBuf::from(skills_dir),
        })
    }

    /// Generate atomic skill registry JSON for LLM
    ///
    /// This generates a JSON registry of all available atomic skills
    /// that the LLM can use to decide which skill to call.
    fn get_atomic_skills_registry(&self) -> String {
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
                    })
                })
            })
            .collect();
        serde_json::to_string_pretty(&registry).unwrap_or_else(|_| "[]".to_string())
    }

    /// Build system prompt for natural language processing
    fn build_natural_language_prompt(&self) -> String {
        let registry_json = self.get_atomic_skills_registry();
        SYSTEM_PROMPT_TEMPLATE.replace("{}", &registry_json)
    }

    /// Parse LLM response into execution instruction
    fn handle_llm_response(&self, response: &str) -> anyhow::Result<ExecutionInstruction> {
        let json_str = Self::extract_json(response);
        let value: Value = serde_json::from_str(&json_str)?;
        if let Some(message) = value.get("message").and_then(|v| v.as_str()) {
            if value.get("action").and_then(|v| v.as_str()) == Some("done") {
                return Ok(ExecutionInstruction::Done(message.to_string()));
            }
        }
        if let Some(mode) = value.get("mode").and_then(|v| v.as_str()) {
            if mode == "batch" {
                if let Some(steps) = value.get("steps").and_then(|v| v.as_array()) {
                    let mut skill_calls = Vec::new();
                    for step in steps {
                        let call: crate::executors::SkillCall =
                            serde_json::from_value(step.clone())?;
                        skill_calls.push(call);
                    }
                    return Ok(ExecutionInstruction::Batch(skill_calls));
                }
            }
        }
        if let Ok(call) = serde_json::from_value(value) {
            return Ok(ExecutionInstruction::Single(call));
        }
        anyhow::bail!("Unable to parse LLM response: {}", response)
    }

    /// Extract JSON from LLM response (handles markdown code blocks)
    pub fn extract_json(text: &str) -> String {
        if let Some(start) = text.find("```json") {
            let after_start = &text[start + 7..];
            if let Some(end) = after_start.find("```") {
                return after_start[..end].trim().to_string();
            }
        }
        if let Some(start) = text.find("```") {
            let after_start = &text[start + 3..];
            if let Some(end) = after_start.find("```") {
                return after_start[..end].trim().to_string();
            }
        }
        if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                return text[start..=end].to_string();
            }
        }
        text.to_string()
    }

    /// Execute a plan (multiple skills in sequence)
    async fn execute_plan(
        &self,
        steps: &[crate::executors::SkillCall],
    ) -> Result<Vec<StepResult>, String> {
        let mut results = Vec::new();
        for step in steps {
            match self.executor.execute(step).await {
                Ok(output) => {
                    results.push(StepResult {
                        skill: step.action.clone(),
                        parameters: step.parameters.clone(),
                        output: output.clone(),
                    });
                }
                Err(e) => {
                    return Err(format!("Skill '{}' failed: {}", step.action, e));
                }
            }
        }
        Ok(results)
    }

    /// Format step results for output
    fn format_step_results(&self, results: &[StepResult]) -> String {
        if results.is_empty() {
            return t!("skill.no_steps_executed").to_string();
        }
        if results.len() == 1 {
            return results[0].output.clone();
        }
        let mut output = format!("{}:\n\n", t!("skill.executed_steps", results.len()));
        for (i, result) in results.iter().enumerate() {
            output.push_str(&format!("{}: {}\n", i + 1, result.output));
        }
        output
    }

    /// Handle natural language input from user
    ///
    /// This function processes user natural language input, uses LLM to
    /// select appropriate atomic skills from the registry, and executes them.
    ///
    /// # Arguments
    /// * `input` - Natural language input from the user
    /// * `session_id` - Optional session ID for conversation history
    ///                  (uses "default" if None)
    ///
    /// # Returns
    /// The response string after processing
    pub async fn handle_natural_language(&self, input: &str, session_id: Option<&str>) -> String {
        let session_id = session_id.unwrap_or("default");
        let input_trimmed = input.trim();
        if input_trimmed == "clear" {
            self.memory.clear_session(session_id);
            return t!("app.conversation_cleared").to_string();
        }
        if input_trimmed == "exit" || input_trimmed == "quit" {
            return "goodbye".to_string();
        }
        if input_trimmed.is_empty() {
            return String::new();
        }
        let history = self.memory.get_history(session_id);
        let mut step_results: Vec<StepResult> = Vec::new();
        let mut final_response = None;
        let max_iterations = 10;
        let mut iteration = 0;
        while iteration < max_iterations {
            iteration += 1;
            let execution_summary = if step_results.is_empty() {
                String::new()
            } else {
                let mut summary = format!("\n## {}\n", t!("skill.previous_executed_steps"));
                for (i, result) in step_results.iter().enumerate() {
                    summary.push_str(&format!("{}. {}\n", i + 1, result.to_string()));
                }
                summary
            };
            let system_prompt = self.build_natural_language_prompt();
            let user_prompt = format!(
                "{}\n\n## {}\n{}\n\n## {}\n{}\n\n{}\n\n## {}\n",
                system_prompt,
                t!("prompt.original_request"),
                input_trimmed,
                t!("prompt.conversation_history"),
                history,
                execution_summary,
                t!("prompt.your_response")
            );
            let llm_response = match self.scheduler.get_llm().generate(&user_prompt).await {
                Ok(resp) => resp,
                Err(e) => {
                    return format!("{}: {}", t!("error.llm_error"), e);
                }
            };
            let instruction = match self.handle_llm_response(&llm_response) {
                Ok(instr) => instr,
                Err(_) => {
                    return llm_response;
                }
            };
            match instruction {
                ExecutionInstruction::Done(message) => {
                    final_response = Some(message);
                    break;
                }
                ExecutionInstruction::Single(call) => match self.executor.execute(&call).await {
                    Ok(output) => {
                        step_results.push(StepResult {
                            skill: call.action.clone(),
                            parameters: call.parameters.clone(),
                            output: output.clone(),
                        });
                    }
                    Err(e) => {
                        final_response = Some(format!(
                            "{} '{}': {}",
                            t!("error.skill_failed"),
                            call.action,
                            e
                        ));
                        break;
                    }
                },
                ExecutionInstruction::Batch(steps) => match self.execute_plan(&steps).await {
                    Ok(results) => {
                        for result in results {
                            step_results.push(result);
                        }
                        let summary = self.format_step_results(&step_results);
                        final_response = Some(summary);
                        break;
                    }
                    Err(e) => {
                        final_response = Some(e);
                        break;
                    }
                },
            }
        }
        if iteration >= max_iterations {
            final_response = Some(t!("error.max_iterations_reached").to_string());
        }
        let final_response = final_response.unwrap_or_else(|| {
            if step_results.is_empty() {
                t!("skill.no_actions_executed").to_string()
            } else {
                self.format_step_results(&step_results)
            }
        });
        self.memory.add_exchange(session_id, input, &final_response);
        final_response
    }

    /// Handle multiple natural language inputs in parallel
    ///
    /// This function processes multiple natural language inputs concurrently.
    /// Each input uses its own session ID or shares the same session.
    ///
    /// # Arguments
    /// * `inputs` - A vector of tuples: `Vec<(String, Option<String>)>`
    ///     - First element: The natural language input text
    ///     - Second element: Optional session ID for conversation history
    ///       (uses "default" if None)
    ///
    /// # Returns
    /// A vector of response strings in the **same order** as the input tasks.
    ///
    /// ## Return Value Structure
    /// - **Success**: Returns the natural language response string
    /// - **Failure**: Returns an error message string (e.g., "LLM error: ...", "Task panic: ...")
    /// - **Empty**: Returns empty string for empty input
    ///
    /// Each element in the returned vector corresponds to the task at the same index.
    /// Errors in one task do not affect other tasks.
    ///
    /// # Example
    /// ```ignore
    /// let inputs = vec![
    ///     ("What is 2+2?".to_string(), Some("user123".to_string())),
    ///     ("Tell me a joke".to_string(), None),
    /// ];
    ///
    /// let results = hippox.handle_natural_language_batch(inputs).await;
    /// ```
    pub async fn handle_natural_language_batch(
        &self,
        inputs: Vec<(String, Option<String>)>,
    ) -> Vec<String> {
        if inputs.is_empty() {
            return Vec::new();
        }
        info!(
            "Processing {} natural language inputs in parallel",
            inputs.len()
        );
        let mut handles = Vec::new();
        for (input, session_id) in inputs {
            let self_clone = self.clone();
            let handle = tokio::spawn(async move {
                self_clone
                    .handle_natural_language(&input, session_id.as_deref())
                    .await
            });
            handles.push(handle);
        }
        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(format!("{}: {}", t!("error.task_panic"), e)),
            }
        }
        results
    }

    /// Handle SKILL.md file execution
    ///
    /// This function loads and executes a SKILL.md file as a predefined workflow.
    /// The skill file is loaded from the skills directory and executed step by step.
    ///
    /// # Arguments
    /// * `skill_name` - Name of the skill (subdirectory name containing SKILL.md)
    /// * `params` - Optional parameters to pass to the skill execution
    ///
    /// # Returns
    /// The execution result as a string
    pub async fn handle_skill_md(
        &self,
        skill_name: &str,
        params: Option<HashMap<String, Value>>,
    ) -> String {
        // Load the SKILL.md file
        let skill_file =
            match SkillLoader::load_by_name(self.skills_dir.to_str().unwrap_or("."), skill_name) {
                Ok(Some(file)) => file,
                Ok(None) => {
                    return format!("{}: {}", t!("error.skill_not_found"), skill_name);
                }
                Err(e) => {
                    return format!("{}: {}", t!("error.load_skill_failed"), e);
                }
            };
        info!("Executing SKILL.md: {}", skill_name);
        let instructions = &skill_file.instructions;
        let registry_json = self.get_atomic_skills_registry();
        let workflow_prompt = format!(
            r#"You are executing a predefined workflow from a SKILL.md file.

## Workflow Instructions
{}

## Available Atomic Skills
{}

## Parameters
{}

## Task
Execute the workflow according to the instructions above. Use the available atomic skills to complete each step.
Respond with the final result of the workflow execution.
"#,
            instructions,
            registry_json,
            serde_json::to_string_pretty(&params.unwrap_or_default()).unwrap_or_default()
        );
        match self.scheduler.get_llm().generate(&workflow_prompt).await {
            Ok(response) => response,
            Err(e) => format!("{}: {}", t!("error.llm_error"), e),
        }
    }

    /// Handle multiple SKILL.md files execution in parallel
    ///
    /// This function executes multiple SKILL.md workflows concurrently.
    /// Each workflow is independent and runs in its own task.
    ///
    /// # Arguments
    /// * `tasks` - A vector of tuples: `Vec<(String, Option<HashMap<String, Value>>)>`
    ///     - First element: The skill name (subdirectory name containing SKILL.md)
    ///     - Second element: Optional parameters to pass to the skill execution
    ///       (None means no parameters)
    ///
    /// # Returns
    /// A vector of result strings in the **same order** as the input tasks.
    ///
    /// ## Return Value Structure
    /// | Case | Return Value Format |
    /// |------|---------------------|
    /// | **Success** | The execution result from the SKILL.md workflow |
    /// | **Skill not found** | `"error.skill_not_found: {skill_name}"` |
    /// | **Load failed** | `"error.load_skill_failed: {error}"` |
    /// | **LLM error** | `"error.llm_error: {error}"` |
    /// | **Task panic** | `"error.task_panic: {error}"` |
    ///
    /// Errors in one task do not affect other tasks.
    /// The returned vector maintains the same index order as the input tasks.
    ///
    /// # Example
    /// ```ignore
    /// let mut email_params = HashMap::new();
    /// email_params.insert("to".to_string(), json!("admin@example.com"));
    ///
    /// let tasks = vec![
    ///     ("daily_report".to_string(), Some(report_params)),
    ///     ("send_email".to_string(), Some(email_params)),
    ///     ("backup_data".to_string(), None),
    /// ];
    ///
    /// let results = hippox.handle_skill_md_batch(tasks).await;
    /// ```
    pub async fn handle_skill_md_batch(
        &self,
        tasks: Vec<(String, Option<HashMap<String, Value>>)>,
    ) -> Vec<String> {
        if tasks.is_empty() {
            return Vec::new();
        }
        info!("Executing {} SKILL.md files in parallel", tasks.len());
        let mut handles = Vec::new();
        for (skill_name, params) in tasks {
            let self_clone = self.clone();
            let handle =
                tokio::spawn(async move { self_clone.handle_skill_md(&skill_name, params).await });
            handles.push(handle);
        }
        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(format!("{}: {}", t!("error.task_panic"), e)),
            }
        }
        results
    }

    /// Clear conversation history for a session
    pub fn clear_conversation(&self, session_id: &str) {
        self.memory.clear_session(session_id);
    }

    /// Clear all conversation histories
    pub fn clear_all_conversations(&self) {
        self.memory.clear_all();
    }

    /// List all available atomic skills
    pub fn list_atomic_skills(&self) -> String {
        let skills = crate::executors::registry::list_skills();
        if skills.is_empty() {
            return t!("skill.no_skills_available").to_string();
        }
        let mut result = String::new();
        for name in skills {
            if let Some(skill) = crate::executors::registry::get_skill(&name) {
                let emoji = match skill.category() {
                    "file" => "📁",
                    "net" => "🌐",
                    "math" => "🔢",
                    "time" => "🕐",
                    "system" => "💻",
                    "db" => "🗄️",
                    "devops" => "🚀",
                    "document" => "📄",
                    "message" => "💬",
                    "task" => "⏰",
                    _ => "⚙️",
                };
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

    /// List all available SKILL.md files in the skills directory
    pub fn list_skill_md_files(&self) -> String {
        match SkillLoader::load_all(self.skills_dir.to_str().unwrap_or(".")) {
            Ok(skills) => {
                if skills.is_empty() {
                    return t!("skill.no_skill_md_available").to_string();
                }
                let mut result = String::new();
                for skill in skills {
                    let emoji = skill
                        .metadata
                        .as_ref()
                        .and_then(|m| m.emoji.as_ref())
                        .map(|e| e.as_str())
                        .unwrap_or("📋");
                    result.push_str(&format!(
                        "   {} - **{}**: {}\n",
                        emoji, skill.name, skill.description
                    ));
                }
                result
            }
            Err(e) => format!("{}: {}", t!("error.list_skills_failed"), e),
        }
    }

    /// Get all loaded atomic skill names
    pub fn get_atomic_skill_names(&self) -> Vec<String> {
        crate::executors::registry::list_skills()
    }

    /// Get all SKILL.md file names
    pub fn get_skill_md_names(&self) -> Vec<String> {
        match SkillLoader::load_all(self.skills_dir.to_str().unwrap_or(".")) {
            Ok(skills) => skills.into_iter().map(|s| s.name).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Check if there are any atomic skills available
    pub fn has_atomic_skills(&self) -> bool {
        !crate::executors::registry::list_skills().is_empty()
    }

    /// Get the skills directory path
    pub fn skills_directory(&self) -> &PathBuf {
        &self.skills_dir
    }

    /// Get the executor
    pub fn executor(&self) -> &Executor {
        &self.executor
    }

    /// Get the scheduler
    pub fn scheduler(&self) -> &SkillScheduler {
        &self.scheduler
    }

    pub fn update_config<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut HippoxConfig),
    {
        crate::config::update_config(f)
    }

    pub fn get_config(&self) -> HippoxConfig {
        crate::config::get_config()
    }
}

/// Execution instruction parsed from LLM response
pub enum ExecutionInstruction {
    Done(String),
    Single(crate::executors::SkillCall),
    Batch(Vec<crate::executors::SkillCall>),
}

#[cfg(test)]
mod tests {
    use super::*;
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
    async fn test_hippox_new_with_env() {
        let temp_dir = tempdir().unwrap();
        let hippox = Hippox::new(
            temp_dir.path().to_str().unwrap(),
            ModelProvider::OpenAI,
            Some("test-api-key".to_string()),
            None,
            ConfigInitMethod::Env,
        )
        .await;
        assert!(hippox.is_ok());
    }

    #[tokio::test]
    async fn test_hippox_new_with_params_json() {
        let temp_dir = tempdir().unwrap();
        let config_json = r#"{"lang": "zh", "provider": "openai"}"#;
        let hippox = Hippox::new(
            temp_dir.path().to_str().unwrap(),
            ModelProvider::OpenAI,
            Some("test-api-key".to_string()),
            None,
            ConfigInitMethod::ParamsJsonStr(config_json.to_string()),
        )
        .await;
        assert!(hippox.is_ok());
        let hippox = hippox.unwrap();
        let config = hippox.get_config();
        assert_eq!(config.lang, "zh");
    }

    #[tokio::test]
    async fn test_list_atomic_skills() {
        let temp_dir = tempdir().unwrap();
        let hippox = Hippox::new(
            temp_dir.path().to_str().unwrap(),
            ModelProvider::OpenAI,
            Some("test-api-key".to_string()),
            None,
            ConfigInitMethod::Env,
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
            Some("test-api-key".to_string()),
            None,
            ConfigInitMethod::Env,
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
            Some("test-api-key".to_string()),
            None,
            ConfigInitMethod::Env,
        )
        .await
        .unwrap();
        hippox.clear_conversation("test-session");
        hippox.clear_all_conversations();
    }

    #[tokio::test]
    async fn test_update_config() {
        let temp_dir = tempdir().unwrap();
        let hippox = Hippox::new(
            temp_dir.path().to_str().unwrap(),
            ModelProvider::OpenAI,
            Some("test-api-key".to_string()),
            None,
            ConfigInitMethod::Env,
        )
        .await
        .unwrap();
        hippox
            .update_config(|config| {
                config.lang = "zh".to_string();
            })
            .unwrap();
        let config = hippox.get_config();
        assert_eq!(config.lang, "zh");
    }

    #[test]
    fn test_extract_json() {
        let text = r#"Some text {"action": "calculator", "parameters": {"input": "2+2"}}"#;
        let json = Hippox::extract_json(text);
        assert!(json.contains("calculator"));
        let text = "```json\n{\"action\": \"test\"}\n```";
        let json = Hippox::extract_json(text);
        assert_eq!(json, "{\"action\": \"test\"}");
        let text = "```\n{\"action\": \"test\"}\n```";
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
                Some("test-api-key".to_string()),
                None,
                ConfigInitMethod::Env,
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
                Some("test-api-key".to_string()),
                None,
                ConfigInitMethod::Env,
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
            Hippox::new(
                path,
                ModelProvider::OpenAI,
                Some("test-api-key".to_string()),
                None,
                ConfigInitMethod::Env,
            )
            .await
            .unwrap()
        });
        assert_eq!(hippox.skills_directory().to_str().unwrap(), path);
    }

    #[test]
    fn test_get_config() {
        let temp_dir = tempdir().unwrap();
        let hippox = tokio::runtime::Runtime::new().unwrap().block_on(async {
            Hippox::new(
                temp_dir.path().to_str().unwrap(),
                ModelProvider::OpenAI,
                Some("test-api-key".to_string()),
                None,
                ConfigInitMethod::Env,
            )
            .await
            .unwrap()
        });
        let config = hippox.get_config();
        assert_eq!(config.lang, "en");
    }
}
