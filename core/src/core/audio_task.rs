//! Audio task definitions and dedicated task pool.
use crate::HippoxResult;
use crate::HippoxStringResult;
use crate::base64_decode_to_file;
use crate::download_to_file;
use langhub::audio::{AudioLLMOptions, AudioModelProvider, AudioUsage};
use langhub::{AudioLLMClient, AudioLLMConfig};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;
/// Status of a single audio generation task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioTaskState {
    /// Task has been created but not started yet.
    Pending,
    /// Task is currently calling the LLM API.
    Running,
    /// Task completed successfully and the audio was saved to disk.
    Succeeded,
    /// Task failed with an error message.
    Failed,
}
/// A single audio generation task record.
#[derive(Debug, Clone)]
pub struct AudioTaskRecord {
    /// Unique task ID.
    pub task_id: String,
    /// Provider used for this task.
    pub provider: AudioModelProvider,
    /// Original prompt.
    pub prompt: String,
    /// Current task state.
    pub state: AudioTaskState,
    /// Local path where the produced audio was saved (on success).
    pub output_path: Option<String>,
    /// Error message (on failure).
    pub error: Option<String>,
    /// Real usage reported by the provider (if any).
    pub usage: Option<AudioUsage>,
}
/// Dedicated task pool for audio generation tasks.
#[derive(Debug, Default)]
pub struct AudioTaskPool {
    tasks: HashMap<String, AudioTaskRecord>,
}
impl AudioTaskPool {
    /// Creates a new empty audio task pool.
    pub fn new() -> Self {
        Self { tasks: HashMap::new() }
    }
    /// Inserts a new task record and returns its ID.
    pub fn insert(&mut self, record: AudioTaskRecord) -> String {
        let id = record.task_id.clone();
        self.tasks.insert(id.clone(), record);
        id
    }
    /// Updates the state of an existing task.
    pub fn set_state(&mut self, task_id: &str, state: AudioTaskState) {
        if let Some(record) = self.tasks.get_mut(task_id) {
            record.state = state;
        }
    }
    /// Marks a task as succeeded and records the output path.
    pub fn set_succeeded(&mut self, task_id: &str, output_path: String) {
        if let Some(record) = self.tasks.get_mut(task_id) {
            record.state = AudioTaskState::Succeeded;
            record.output_path = Some(output_path);
            record.error = None;
        }
    }
    /// Marks a task as failed and records the error message.
    pub fn set_failed(&mut self, task_id: &str, error: String) {
        if let Some(record) = self.tasks.get_mut(task_id) {
            record.state = AudioTaskState::Failed;
            record.error = Some(error);
        }
    }
    /// Records usage for a task.
    pub fn set_usage(&mut self, task_id: &str, usage: AudioUsage) {
        if let Some(record) = self.tasks.get_mut(task_id) {
            record.usage = Some(usage);
        }
    }
    /// Gets a clone of a task record by ID.
    pub fn get(&self, task_id: &str) -> Option<AudioTaskRecord> {
        self.tasks.get(task_id).cloned()
    }
}
/// Global dedicated audio task pool.
pub static AUDIO_TASK_POOL: Lazy<Arc<RwLock<AudioTaskPool>>> = Lazy::new(|| Arc::new(RwLock::new(AudioTaskPool::new())));
/// Get the current state of an audio task.
pub async fn get_audio_task(task_id: &str) -> HippoxResult<AudioTaskRecord> {
    let pool = AUDIO_TASK_POOL.read().await;
    match pool.get(task_id) {
        Some(record) => HippoxResult::ok(record),
        None => HippoxResult::system_error(format!("Audio task not found: {}", task_id)),
    }
}
/// Get the usage recorded for a specific audio task.
pub async fn get_audio_task_usage(task_id: &str) -> HippoxResult<Option<AudioUsage>> {
    let pool = AUDIO_TASK_POOL.read().await;
    match pool.get(task_id) {
        Some(record) => HippoxResult::ok(record.usage),
        None => HippoxResult::system_error(format!("Audio task not found: {}", task_id)),
    }
}
/// Internal implementation of an audio generation task.
pub(crate) async fn run_audio_task(
    provider: AudioModelProvider,
    api_key: String,
    prompt: String,
    options: Option<AudioLLMOptions>,
    base_url: Option<String>,
    output_filename: Option<String>,
    output_path: String,
) -> HippoxStringResult {
    let task_id = Uuid::new_v4().to_string();
    info!("Submitting audio task {}: provider={:?}, prompt_len={}", task_id, provider, prompt.len());
    {
        let mut pool = AUDIO_TASK_POOL.write().await;
        pool.insert(AudioTaskRecord {
            task_id: task_id.clone(),
            provider,
            prompt: prompt.clone(),
            state: AudioTaskState::Pending,
            output_path: None,
            error: None,
            usage: None,
        });
    }
    {
        let mut pool = AUDIO_TASK_POOL.write().await;
        pool.set_state(&task_id, AudioTaskState::Running);
    }
    let mut config = AudioLLMConfig::new();
    config = match provider {
        AudioModelProvider::QwenTts => {
            let mut c = config.qwen_tts(api_key.clone());
            if let Some(base) = &base_url {
                c.qwen_tts_base_url = Some(base.clone());
            }
            c
        }
        AudioModelProvider::SeedAudio => {
            let mut c = config.seed_audio(api_key.clone());
            if let Some(base) = &base_url {
                c.seed_audio_base_url = Some(base.clone());
            }
            c
        }
        AudioModelProvider::StepAudio => {
            let mut c = config.step_audio(api_key.clone());
            if let Some(base) = &base_url {
                c.step_audio_base_url = Some(base.clone());
            }
            c
        }
        AudioModelProvider::GeminiTts => {
            let mut c = config.gemini_tts(api_key.clone());
            if let Some(base) = &base_url {
                c.gemini_tts_base_url = Some(base.clone());
            }
            c
        }
        AudioModelProvider::ElevenLabs => {
            let mut c = config.elevenlabs(api_key.clone());
            if let Some(base) = &base_url {
                c.elevenlabs_base_url = Some(base.clone());
            }
            c
        }
        AudioModelProvider::Lyria => {
            let mut c = config.lyria(api_key.clone());
            if let Some(base) = &base_url {
                c.lyria_base_url = Some(base.clone());
            }
            c
        }
        AudioModelProvider::Suno => {
            let mut c = config.suno(api_key.clone());
            if let Some(base) = &base_url {
                c.suno_base_url = Some(base.clone());
            }
            c
        }
        AudioModelProvider::StableAudio => {
            let mut c = config.stable_audio(api_key.clone());
            if let Some(base) = &base_url {
                c.stable_audio_base_url = Some(base.clone());
            }
            c
        }
    };
    let client = match AudioLLMClient::new_with_config(provider, &config) {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to create audio client: {}", e);
            warn!("{}", err_msg);
            let mut pool = AUDIO_TASK_POOL.write().await;
            pool.set_failed(&task_id, err_msg.clone());
            return HippoxResult::system_error(err_msg);
        }
    };
    let result = match options {
        Some(opts) => client.generate_with_options(&prompt, opts).await,
        None => client.generate(&prompt).await,
    };
    let result = match result {
        Ok(r) => r,
        Err(e) => {
            let err_msg = format!("Audio generation failed: {}", e);
            warn!("{}", err_msg);
            let mut pool = AUDIO_TASK_POOL.write().await;
            pool.set_failed(&task_id, err_msg.clone());
            return HippoxResult::system_error(err_msg);
        }
    };
    if let Some(usage) = result.extract_usage() {
        let mut pool = AUDIO_TASK_POOL.write().await;
        pool.set_usage(&task_id, usage);
    }
    let default_ext = result.format.clone().unwrap_or_else(|| "mp3".to_string());
    let filename = output_filename.unwrap_or_else(|| format!("{}.{}", task_id, default_ext));
    let full_path = std::path::Path::new(&output_path).join(&filename);
    let full_path_str = full_path.to_string_lossy().to_string();
    if let Some(parent) = full_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            let err_msg = format!("Failed to create output directory: {}", e);
            warn!("{}", err_msg);
            let mut pool = AUDIO_TASK_POOL.write().await;
            pool.set_failed(&task_id, err_msg.clone());
            return HippoxResult::system_error(err_msg);
        }
    }
    if let Some(url) = &result.audio_url {
        if let Err(e) = download_to_file(url, &full_path).await {
            let err_msg = format!("Failed to download audio from {}: {}", url, e);
            warn!("{}", err_msg);
            let mut pool = AUDIO_TASK_POOL.write().await;
            pool.set_failed(&task_id, err_msg.clone());
            return HippoxResult::system_error(err_msg);
        }
        info!("Audio task {} saved to {}", task_id, full_path_str);
        let mut pool = AUDIO_TASK_POOL.write().await;
        pool.set_succeeded(&task_id, full_path_str.clone());
        HippoxResult::ok(full_path_str)
    } else if let Some(b64) = &result.audio_base64 {
        match base64_decode_to_file(b64, &full_path) {
            Ok(_) => {
                info!("Audio task {} saved (base64) to {}", task_id, full_path_str);
                let mut pool = AUDIO_TASK_POOL.write().await;
                pool.set_succeeded(&task_id, full_path_str.clone());
                HippoxResult::ok(full_path_str)
            }
            Err(e) => {
                let err_msg = format!("Failed to decode base64 audio: {}", e);
                warn!("{}", err_msg);
                let mut pool = AUDIO_TASK_POOL.write().await;
                pool.set_failed(&task_id, err_msg.clone());
                HippoxResult::system_error(err_msg)
            }
        }
    } else {
        let err_msg = "Audio generation returned no audio URL or base64 payload".to_string();
        warn!("{}", err_msg);
        let mut pool = AUDIO_TASK_POOL.write().await;
        pool.set_failed(&task_id, err_msg.clone());
        HippoxResult::system_error(err_msg)
    }
}
#[cfg(test)]
mod audio_task_tests {
    use super::*;
    #[test]
    fn test_audio_task_pool_new_is_empty() {
        let pool = AudioTaskPool::new();
        assert!(pool.get("nonexistent").is_none());
    }
    #[test]
    fn test_audio_task_pool_insert_and_get() {
        let mut pool = AudioTaskPool::new();
        let record = AudioTaskRecord {
            task_id: "audio-1".to_string(),
            provider: AudioModelProvider::QwenTts,
            prompt: "hello".to_string(),
            state: AudioTaskState::Pending,
            output_path: None,
            error: None,
            usage: None,
        };
        let id = pool.insert(record);
        assert_eq!(id, "audio-1");
        let fetched = pool.get("audio-1").unwrap();
        assert_eq!(fetched.task_id, "audio-1");
        assert_eq!(fetched.provider, AudioModelProvider::QwenTts);
        assert_eq!(fetched.state, AudioTaskState::Pending);
    }
    #[test]
    fn test_audio_task_pool_set_state() {
        let mut pool = AudioTaskPool::new();
        pool.insert(AudioTaskRecord {
            task_id: "audio-2".to_string(),
            provider: AudioModelProvider::ElevenLabs,
            prompt: "hello".to_string(),
            state: AudioTaskState::Pending,
            output_path: None,
            error: None,
            usage: None,
        });
        pool.set_state("audio-2", AudioTaskState::Running);
        assert_eq!(pool.get("audio-2").unwrap().state, AudioTaskState::Running);
    }
    #[test]
    fn test_audio_task_pool_set_succeeded() {
        let mut pool = AudioTaskPool::new();
        pool.insert(AudioTaskRecord {
            task_id: "audio-3".to_string(),
            provider: AudioModelProvider::Suno,
            prompt: "hello".to_string(),
            state: AudioTaskState::Running,
            output_path: None,
            error: Some("previous error".to_string()),
            usage: None,
        });
        pool.set_succeeded("audio-3", "/tmp/out.mp3".to_string());
        let record = pool.get("audio-3").unwrap();
        assert_eq!(record.state, AudioTaskState::Succeeded);
        assert_eq!(record.output_path, Some("/tmp/out.mp3".to_string()));
        assert!(record.error.is_none());
    }
    #[test]
    fn test_audio_task_pool_set_failed() {
        let mut pool = AudioTaskPool::new();
        pool.insert(AudioTaskRecord {
            task_id: "audio-4".to_string(),
            provider: AudioModelProvider::Lyria,
            prompt: "hello".to_string(),
            state: AudioTaskState::Running,
            output_path: None,
            error: None,
            usage: None,
        });
        pool.set_failed("audio-4", "boom".to_string());
        let record = pool.get("audio-4").unwrap();
        assert_eq!(record.state, AudioTaskState::Failed);
        assert_eq!(record.error, Some("boom".to_string()));
    }
    #[test]
    fn test_audio_task_pool_set_state_missing() {
        let mut pool = AudioTaskPool::new();
        pool.set_state("nonexistent", AudioTaskState::Running);
        assert!(pool.get("nonexistent").is_none());
    }
    #[test]
    fn test_audio_task_pool_set_usage() {
        let mut pool = AudioTaskPool::new();
        pool.insert(AudioTaskRecord {
            task_id: "audio-5".to_string(),
            provider: AudioModelProvider::ElevenLabs,
            prompt: "hello".to_string(),
            state: AudioTaskState::Running,
            output_path: None,
            error: None,
            usage: None,
        });
        let usage = AudioUsage { billed_seconds: 5.0, billed_characters: Some(120), estimated_cost_usd: Some(0.02) };
        pool.set_usage("audio-5", usage.clone());
        let record = pool.get("audio-5").unwrap();
        assert_eq!(record.usage.unwrap().billed_characters, Some(120));
    }
    #[tokio::test]
    async fn test_get_audio_task_not_found() {
        let result = get_audio_task("definitely-not-a-real-task-id").await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_get_audio_task_found() {
        let task_id = format!("unit-test-{}", Uuid::new_v4());
        {
            let mut pool = AUDIO_TASK_POOL.write().await;
            pool.insert(AudioTaskRecord {
                task_id: task_id.clone(),
                provider: AudioModelProvider::GeminiTts,
                prompt: "unit test".to_string(),
                state: AudioTaskState::Pending,
                output_path: None,
                error: None,
                usage: None,
            });
        }
        let result = get_audio_task(&task_id).await;
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.task_id, task_id);
        assert_eq!(record.provider, AudioModelProvider::GeminiTts);
        // Cleanup
        {
            let mut pool = AUDIO_TASK_POOL.write().await;
            pool.tasks.remove(&task_id);
        }
    }
    #[tokio::test]
    async fn test_get_audio_task_usage_none() {
        let task_id = format!("unit-test-usage-{}", Uuid::new_v4());
        {
            let mut pool = AUDIO_TASK_POOL.write().await;
            pool.insert(AudioTaskRecord {
                task_id: task_id.clone(),
                provider: AudioModelProvider::QwenTts,
                prompt: "usage test".to_string(),
                state: AudioTaskState::Succeeded,
                output_path: Some("/tmp/x.mp3".to_string()),
                error: None,
                usage: None,
            });
        }
        let result = get_audio_task_usage(&task_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        // Cleanup
        {
            let mut pool = AUDIO_TASK_POOL.write().await;
            pool.tasks.remove(&task_id);
        }
    }
    #[tokio::test]
    async fn test_get_audio_task_usage_some() {
        let task_id = format!("unit-test-usage-some-{}", Uuid::new_v4());
        {
            let mut pool = AUDIO_TASK_POOL.write().await;
            pool.insert(AudioTaskRecord {
                task_id: task_id.clone(),
                provider: AudioModelProvider::QwenTts,
                prompt: "usage test".to_string(),
                state: AudioTaskState::Succeeded,
                output_path: Some("/tmp/x.mp3".to_string()),
                error: None,
                usage: Some(AudioUsage { billed_seconds: 5.0, billed_characters: Some(999), estimated_cost_usd: Some(0.05) }),
            });
        }
        let result = get_audio_task_usage(&task_id).await;
        assert!(result.is_ok());
        let usage = result.unwrap().unwrap();
        assert_eq!(usage.billed_characters, Some(999));
        // Cleanup
        {
            let mut pool = AUDIO_TASK_POOL.write().await;
            pool.tasks.remove(&task_id);
        }
    }
}
