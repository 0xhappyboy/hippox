use crate::executors::Executor;
use crate::global::Config;
use crate::i18n;
use crate::protocols;
use crate::skill_loader::SkillLoader;
use crate::skill_scheduler::SkillScheduler;
use crate::t;
use crate::types::ProcessResult;
use langhub::LLMClient;
use langhub::types::{ChatMessage, ModelProvider};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::info;

/// Service configuration for which protocols to enable
#[derive(Clone)]
pub struct ServiceConfig {
    pub enable_cli: bool,
    pub enable_tcp: bool,
    pub enable_http: bool,
    pub enable_websocket: bool,
    pub enable_grpc: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            enable_cli: true,
            enable_tcp: false,
            enable_http: false,
            enable_websocket: false,
            enable_grpc: false,
        }
    }
}

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

#[derive(Clone)]
pub struct Hippox {
    scheduler: SkillScheduler,
    executor: Executor,
    conversations: Arc<RwLock<HashMap<String, Vec<String>>>>,
    skills_dir: String,
}

impl Hippox {
    pub async fn new(
        skills_dir: &str,
        provider: ModelProvider,
        lang: &str,
    ) -> anyhow::Result<Self> {
        i18n::set_language(lang);
        info!("Loading skills from: {}", skills_dir);
        let skills = SkillLoader::load_all(skills_dir)?;
        info!("🦛 Loaded {} skills", skills.len());
        for skill in &skills {
            info!("   - {}: {}", skill.name, skill.description);
        }
        let llm = LLMClient::new(provider)?;
        let scheduler = SkillScheduler::new(skills, llm);
        let executor = Executor::new();
        Ok(Self {
            scheduler,
            executor,
            conversations: Arc::new(RwLock::new(HashMap::new())),
            skills_dir: skills_dir.to_string(),
        })
    }

    /// Start the core with the given service configuration
    pub async fn start(self, config: ServiceConfig) -> anyhow::Result<()> {
        let core = Arc::new(self);
        if config.enable_cli {
            let core_cli = core.clone();
            tokio::spawn(async move {
                info!("Starting CLI interface");
                if let Err(e) = protocols::cli::run_cli(core_cli).await {
                    eprintln!("CLI error: {}", e);
                }
            });
        }
        if config.enable_tcp {
            let core_tcp = core.clone();
            tokio::spawn(async move {
                let addr = Config::tcp_address();
                info!("Starting TCP server on {}", addr);
                if let Err(e) = protocols::tcp::run_tcp_server(core_tcp, &addr).await {
                    eprintln!("TCP server error: {}", e);
                }
            });
        }
        if config.enable_http {
            let core_http = core.clone();
            tokio::spawn(async move {
                let addr = Config::http_address();
                info!("Starting HTTP server on http://{}", addr);
                if let Err(e) = protocols::http::run_http_server(core_http, &addr).await {
                    eprintln!("HTTP server error: {}", e);
                }
            });
        }
        if config.enable_websocket {
            let core_ws = core.clone();
            tokio::spawn(async move {
                let addr = Config::websocket_address();
                info!("Starting WebSocket server on ws://{}", addr);
                if let Err(e) = protocols::websocket::run_websocket_server(core_ws, &addr).await {
                    eprintln!("WebSocket server error: {}", e);
                }
            });
        }
        tokio::signal::ctrl_c().await?;
        info!("Shutting down...");
        Ok(())
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

    /// Build system prompt word for multi-step execution
    fn build_multi_step_prompt_word(&self, skill_registry: &str) -> String {
        let prompt = r#"You are an AI assistant that can execute skills/tools.

## Available Skills (JSON Registry)
"#
        .to_string()
            + skill_registry
            + r#"

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
        prompt.to_string()
    }

    /// Parse LLM response into execution instruction
    fn parse_llm_response(&self, response: &str) -> anyhow::Result<ExecutionInstruction> {
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

    /// Extract JSON from LLM response
    fn extract_json(text: &str) -> String {
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

    /// Main process function with multi-step execution support
    pub async fn process(&self, input: &str) -> ProcessResult {
        let session_id = "default".to_string();
        let registry_json =
            match SkillLoader::create_skills_registry_table_json_str(&self.skills_dir) {
                Ok(reg) => reg,
                Err(e) => {
                    return ProcessResult {
                        response: format!("Failed to load skills: {}", e),
                        matched: false,
                        skill_name: None,
                    };
                }
            };
        let input_trimmed = input.trim();
        if input_trimmed == "clear" {
            let mut conversations = self.conversations.write().unwrap();
            conversations.remove(&session_id);
            return ProcessResult {
                response: t!("app.conversation_cleared").to_string(),
                matched: true,
                skill_name: None,
            };
        }
        if input_trimmed == "exit" || input_trimmed == "quit" {
            return ProcessResult {
                response: "goodbye".to_string(),
                matched: true,
                skill_name: None,
            };
        }
        if input_trimmed.is_empty() {
            return ProcessResult {
                response: String::new(),
                matched: false,
                skill_name: None,
            };
        }
        let history = {
            let conversations = self.conversations.read().unwrap();
            conversations
                .get(&session_id)
                .map(|h| h.join("\n"))
                .unwrap_or_default()
        };
        let mut step_results: Vec<StepResult> = Vec::new();
        let mut current_input = input_trimmed.to_string();
        let mut final_response = None;
        let max_iterations = 10;
        let mut iteration = 0;
        while iteration < max_iterations {
            iteration += 1;
            let execution_summary = if step_results.is_empty() {
                String::new()
            } else {
                let mut summary = "\n## Previously Executed Steps\n".to_string();
                for (i, result) in step_results.iter().enumerate() {
                    summary.push_str(&format!("{}. {}\n", i + 1, result.to_string()));
                }
                summary
            };
            let system_prompt = self.build_multi_step_prompt_word(&registry_json);
            let user_prompt = format!(
                "{}\n\n## Original Request\n{}\n\n## Conversation History\n{}\n\n{}\n\n## Your Response\n",
                system_prompt, input_trimmed, history, execution_summary
            );
            let llm_response = match self.scheduler.get_llm().generate(&user_prompt).await {
                Ok(resp) => resp,
                Err(e) => {
                    return ProcessResult {
                        response: format!("LLM error: {}", e),
                        matched: false,
                        skill_name: None,
                    };
                }
            };
            let instruction = match self.parse_llm_response(&llm_response) {
                Ok(instr) => instr,
                Err(e) => {
                    return ProcessResult {
                        response: llm_response,
                        matched: false,
                        skill_name: None,
                    };
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
                        final_response =
                            Some(format!("Skill '{}' execution failed: {}", call.action, e));
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
            final_response = Some("Max iterations reached. Task incomplete.".to_string());
        }
        let final_response = final_response.unwrap_or_else(|| {
            if step_results.is_empty() {
                "No actions were executed.".to_string()
            } else {
                self.format_step_results(&step_results)
            }
        });
        let mut conversations = self.conversations.write().unwrap();
        let hist = conversations.entry(session_id).or_default();
        hist.push(format!("User: {}", input));
        hist.push(format!("Assistant: {}", final_response));
        ProcessResult {
            response: final_response,
            matched: !step_results.is_empty(),
            skill_name: step_results.last().map(|r| r.skill.clone()),
        }
    }

    fn format_step_results(&self, results: &[StepResult]) -> String {
        if results.is_empty() {
            return "No steps executed.".to_string();
        }
        if results.len() == 1 {
            return results[0].output.clone();
        }
        let mut output = format!("Executed {} steps:\n\n", results.len());
        for (i, result) in results.iter().enumerate() {
            output.push_str(&format!("Step {}: {}\n", i + 1, result.output));
        }
        output
    }

    pub fn list_skills(&self) -> String {
        self.scheduler.list_skills()
    }
}

/// Execution instruction parsed from LLM response
pub enum ExecutionInstruction {
    Done(String),
    Single(crate::executors::SkillCall),
    Batch(Vec<crate::executors::SkillCall>),
}
