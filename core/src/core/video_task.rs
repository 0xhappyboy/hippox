//! Video task definitions and dedicated task pool.
//!
//! # Entry point
//! The public entry is `Hippox::submit_video_task` in `hippox.rs`, which calls
//! `run_video_task` in this module.
//!
//! # Design
//! - Uses a dedicated `VIDEO_TASK_POOL` (independent from the general `TASK_POOL`).
//! - Directly calls `VideoLLMClient` from `langhub`.
//! - Downloads the produced video to `output_path` (or `output_path/output_filename`
//!   when a custom filename is provided).
//! - Records real token usage (if the provider returns it) into the task record,
//!   and accumulates it into the global `MEDIA_TOKEN_COUNT`.
use crate::HippoxResult;
use crate::HippoxStringResult;
use crate::base64_decode_to_file;
use crate::download_to_file;
use langhub::types::Result as LangHubResult;
use langhub::video::VideoUsage;
use langhub::video::{VideoLLMOptions, VideoModelProvider};
use langhub::{VideoLLMClient, VideoLLMConfig};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
/// Global media token count (video + image), accumulated from real provider usage.
pub static MEDIA_TOKEN_COUNT: AtomicU64 = AtomicU64::new(0);
/// Status of a single video generation task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoTaskState {
    /// Task has been created but not started yet.
    Pending,
    /// Task is currently calling the LLM API.
    Running,
    /// Task completed successfully and the video was saved to disk.
    Succeeded,
    /// Task failed with an error message.
    Failed,
}
/// A single video generation task record.
#[derive(Debug, Clone)]
pub struct VideoTaskRecord {
    /// Unique task ID.
    pub task_id: String,
    /// Provider used for this task.
    pub provider: VideoModelProvider,
    /// Original prompt.
    pub prompt: String,
    /// Current task state.
    pub state: VideoTaskState,
    /// Local path where the produced video was saved (on success).
    pub output_path: Option<String>,
    /// Error message (on failure).
    pub error: Option<String>,
    /// Real usage reported by the provider (if any).
    pub usage: Option<VideoUsage>,
}
/// Dedicated task pool for video generation tasks.
#[derive(Debug, Default)]
pub struct VideoTaskPool {
    tasks: HashMap<String, VideoTaskRecord>,
}
impl VideoTaskPool {
    /// Creates a new empty video task pool.
    pub fn new() -> Self {
        Self { tasks: HashMap::new() }
    }
    /// Inserts a new task record and returns its ID.
    pub fn insert(&mut self, record: VideoTaskRecord) -> String {
        let id = record.task_id.clone();
        self.tasks.insert(id.clone(), record);
        id
    }
    /// Updates the state of an existing task.
    pub fn set_state(&mut self, task_id: &str, state: VideoTaskState) {
        if let Some(record) = self.tasks.get_mut(task_id) {
            record.state = state;
        }
    }
    /// Marks a task as succeeded and records the output path.
    pub fn set_succeeded(&mut self, task_id: &str, output_path: String) {
        if let Some(record) = self.tasks.get_mut(task_id) {
            record.state = VideoTaskState::Succeeded;
            record.output_path = Some(output_path);
            record.error = None;
        }
    }
    /// Marks a task as failed and records the error message.
    pub fn set_failed(&mut self, task_id: &str, error: String) {
        if let Some(record) = self.tasks.get_mut(task_id) {
            record.state = VideoTaskState::Failed;
            record.error = Some(error);
        }
    }
    /// Records usage for a task.
    pub fn set_usage(&mut self, task_id: &str, usage: VideoUsage) {
        if let Some(record) = self.tasks.get_mut(task_id) {
            record.usage = Some(usage);
        }
    }
    /// Gets a clone of a task record by ID.
    pub fn get(&self, task_id: &str) -> Option<VideoTaskRecord> {
        self.tasks.get(task_id).cloned()
    }
}
/// Global dedicated video task pool.
pub static VIDEO_TASK_POOL: Lazy<Arc<RwLock<VideoTaskPool>>> = Lazy::new(|| Arc::new(RwLock::new(VideoTaskPool::new())));
/// Get the current state of a video task.
pub async fn get_video_task(task_id: &str) -> HippoxResult<VideoTaskRecord> {
    let pool = VIDEO_TASK_POOL.read().await;
    match pool.get(task_id) {
        Some(record) => HippoxResult::ok(record),
        None => HippoxResult::system_error(format!("Video task not found: {}", task_id)),
    }
}
/// Get the usage recorded for a specific video task.
pub async fn get_video_task_usage(task_id: &str) -> HippoxResult<Option<VideoUsage>> {
    let pool = VIDEO_TASK_POOL.read().await;
    match pool.get(task_id) {
        Some(record) => HippoxResult::ok(record.usage),
        None => HippoxResult::system_error(format!("Video task not found: {}", task_id)),
    }
}
/// Get the current global media token count (video + image).
pub fn get_media_token_count() -> u64 {
    MEDIA_TOKEN_COUNT.load(Ordering::Relaxed)
}
/// Internal implementation of a video generation task.
pub(crate) async fn run_video_task(
    provider: VideoModelProvider,
    api_key: String,
    prompt: String,
    options: Option<VideoLLMOptions>,
    base_url: Option<String>,
    output_filename: Option<String>,
    output_path: String,
) -> HippoxStringResult {
    let task_id = Uuid::new_v4().to_string();
    info!("Submitting video task {}: provider={:?}, prompt_len={}", task_id, provider, prompt.len());
    {
        let mut pool = VIDEO_TASK_POOL.write().await;
        pool.insert(VideoTaskRecord {
            task_id: task_id.clone(),
            provider,
            prompt: prompt.clone(),
            state: VideoTaskState::Pending,
            output_path: None,
            error: None,
            usage: None,
        });
    }
    {
        let mut pool = VIDEO_TASK_POOL.write().await;
        pool.set_state(&task_id, VideoTaskState::Running);
    }
    let mut config = VideoLLMConfig::new();
    config = match provider {
        VideoModelProvider::Seedance => {
            let mut c = config.seedance(api_key.clone());
            if let Some(base) = &base_url {
                c.seedance_base_url = Some(base.clone());
            }
            c
        }
        VideoModelProvider::Wan => {
            let mut c = config.wan(api_key.clone());
            if let Some(base) = &base_url {
                c.wan_base_url = Some(base.clone());
            }
            c
        }
        VideoModelProvider::Kling => {
            let mut c = config.kling(api_key.clone(), api_key.clone());
            if let Some(base) = &base_url {
                c.kling_base_url = Some(base.clone());
            }
            c
        }
        VideoModelProvider::Veo => {
            let mut c = config.veo(api_key.clone());
            if let Some(base) = &base_url {
                c.veo_base_url = Some(base.clone());
            }
            c
        }
        VideoModelProvider::Runway => {
            let mut c = config.runway(api_key.clone());
            if let Some(base) = &base_url {
                c.runway_base_url = Some(base.clone());
            }
            c
        }
        VideoModelProvider::MiniMaxH3 => {
            let mut c = config.minimax_h3(api_key.clone(), api_key.clone());
            if let Some(base) = &base_url {
                c.minimax_h3_base_url = Some(base.clone());
            }
            c
        }
        VideoModelProvider::HappyHorse => {
            let mut c = config.happyhorse(api_key.clone());
            if let Some(base) = &base_url {
                c.happyhorse_base_url = Some(base.clone());
            }
            c
        }
        VideoModelProvider::Ltx => {
            let mut c = config.ltx(api_key.clone());
            if let Some(base) = &base_url {
                c.ltx_base_url = Some(base.clone());
            }
            c
        }
        VideoModelProvider::GrokImagine => {
            let mut c = config.grok(api_key.clone());
            if let Some(base) = &base_url {
                c.grok_base_url = Some(base.clone());
            }
            c
        }
        VideoModelProvider::Pruna => {
            let mut c = config.pruna(api_key.clone());
            if let Some(base) = &base_url {
                c.pruna_base_url = Some(base.clone());
            }
            c
        }
        VideoModelProvider::GeminiOmniFlash => {
            let mut c = config.gemini(api_key.clone());
            if let Some(base) = &base_url {
                c.gemini_base_url = Some(base.clone());
            }
            c
        }
    };
    let client = match VideoLLMClient::new_with_config(provider, &config) {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to create video client: {}", e);
            warn!("{}", err_msg);
            let mut pool = VIDEO_TASK_POOL.write().await;
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
            let err_msg = format!("Video generation failed: {}", e);
            warn!("{}", err_msg);
            let mut pool = VIDEO_TASK_POOL.write().await;
            pool.set_failed(&task_id, err_msg.clone());
            return HippoxResult::system_error(err_msg);
        }
    };
    // Record real usage into the task record and the global media counter.
    if let Some(usage) = result.extract_usage() {
        let tokens = usage.billed_tokens.unwrap_or(0);
        if tokens > 0 {
            MEDIA_TOKEN_COUNT.fetch_add(tokens, Ordering::Relaxed);
        }
        let mut pool = VIDEO_TASK_POOL.write().await;
        pool.set_usage(&task_id, usage);
    }
    let filename = output_filename.unwrap_or_else(|| format!("{}.mp4", task_id));
    let full_path = std::path::Path::new(&output_path).join(&filename);
    let full_path_str = full_path.to_string_lossy().to_string();
    if let Some(parent) = full_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            let err_msg = format!("Failed to create output directory: {}", e);
            warn!("{}", err_msg);
            let mut pool = VIDEO_TASK_POOL.write().await;
            pool.set_failed(&task_id, err_msg.clone());
            return HippoxResult::system_error(err_msg);
        }
    }
    if let Some(url) = &result.video_url {
        if let Err(e) = download_to_file(url, &full_path).await {
            let err_msg = format!("Failed to download video from {}: {}", url, e);
            warn!("{}", err_msg);
            let mut pool = VIDEO_TASK_POOL.write().await;
            pool.set_failed(&task_id, err_msg.clone());
            return HippoxResult::system_error(err_msg);
        }
        info!("Video task {} saved to {}", task_id, full_path_str);
        let mut pool = VIDEO_TASK_POOL.write().await;
        pool.set_succeeded(&task_id, full_path_str.clone());
        HippoxResult::ok(full_path_str)
    } else if let Some(b64) = &result.video_base64 {
        match base64_decode_to_file(b64, &full_path) {
            Ok(_) => {
                info!("Video task {} saved (base64) to {}", task_id, full_path_str);
                let mut pool = VIDEO_TASK_POOL.write().await;
                pool.set_succeeded(&task_id, full_path_str.clone());
                HippoxResult::ok(full_path_str)
            }
            Err(e) => {
                let err_msg = format!("Failed to decode base64 video: {}", e);
                warn!("{}", err_msg);
                let mut pool = VIDEO_TASK_POOL.write().await;
                pool.set_failed(&task_id, err_msg.clone());
                HippoxResult::system_error(err_msg)
            }
        }
    } else {
        let err_msg = "Video generation returned no video URL or base64 payload".to_string();
        warn!("{}", err_msg);
        let mut pool = VIDEO_TASK_POOL.write().await;
        pool.set_failed(&task_id, err_msg.clone());
        HippoxResult::system_error(err_msg)
    }
}
#[cfg(test)]
mod video_task_tests {
    use super::*;
    /// Verifies that a new VideoTaskPool is empty.
    #[test]
    fn test_video_task_pool_new_is_empty() {
        let pool = VideoTaskPool::new();
        assert!(pool.get("nonexistent").is_none());
    }
    /// Verifies insert / get round-trip on VideoTaskPool.
    #[test]
    fn test_video_task_pool_insert_and_get() {
        let mut pool = VideoTaskPool::new();
        let record = VideoTaskRecord {
            task_id: "task-1".to_string(),
            provider: VideoModelProvider::Seedance,
            prompt: "a cat".to_string(),
            state: VideoTaskState::Pending,
            output_path: None,
            error: None,
            usage: None,
        };
        let id = pool.insert(record);
        assert_eq!(id, "task-1");
        let fetched = pool.get("task-1").unwrap();
        assert_eq!(fetched.task_id, "task-1");
        assert_eq!(fetched.provider, VideoModelProvider::Seedance);
        assert_eq!(fetched.state, VideoTaskState::Pending);
    }
    /// Verifies set_state updates the task state.
    #[test]
    fn test_video_task_pool_set_state() {
        let mut pool = VideoTaskPool::new();
        pool.insert(VideoTaskRecord {
            task_id: "task-2".to_string(),
            provider: VideoModelProvider::Wan,
            prompt: "a dog".to_string(),
            state: VideoTaskState::Pending,
            output_path: None,
            error: None,
            usage: None,
        });
        pool.set_state("task-2", VideoTaskState::Running);
        assert_eq!(pool.get("task-2").unwrap().state, VideoTaskState::Running);
    }
    /// Verifies set_succeeded records the output path and clears error.
    #[test]
    fn test_video_task_pool_set_succeeded() {
        let mut pool = VideoTaskPool::new();
        pool.insert(VideoTaskRecord {
            task_id: "task-3".to_string(),
            provider: VideoModelProvider::Kling,
            prompt: "a bird".to_string(),
            state: VideoTaskState::Running,
            output_path: None,
            error: Some("previous error".to_string()),
            usage: None,
        });
        pool.set_succeeded("task-3", "/tmp/out.mp4".to_string());
        let record = pool.get("task-3").unwrap();
        assert_eq!(record.state, VideoTaskState::Succeeded);
        assert_eq!(record.output_path, Some("/tmp/out.mp4".to_string()));
        assert!(record.error.is_none());
    }
    /// Verifies set_failed records the error message.
    #[test]
    fn test_video_task_pool_set_failed() {
        let mut pool = VideoTaskPool::new();
        pool.insert(VideoTaskRecord {
            task_id: "task-4".to_string(),
            provider: VideoModelProvider::Veo,
            prompt: "a fish".to_string(),
            state: VideoTaskState::Running,
            output_path: None,
            error: None,
            usage: None,
        });
        pool.set_failed("task-4", "boom".to_string());
        let record = pool.get("task-4").unwrap();
        assert_eq!(record.state, VideoTaskState::Failed);
        assert_eq!(record.error, Some("boom".to_string()));
    }
    /// Verifies set_state on a missing task is a no-op.
    #[test]
    fn test_video_task_pool_set_state_missing() {
        let mut pool = VideoTaskPool::new();
        pool.set_state("nonexistent", VideoTaskState::Running);
        assert!(pool.get("nonexistent").is_none());
    }
    /// Verifies set_usage records the usage on the task.
    #[test]
    fn test_video_task_pool_set_usage() {
        let mut pool = VideoTaskPool::new();
        pool.insert(VideoTaskRecord {
            task_id: "task-5".to_string(),
            provider: VideoModelProvider::Seedance,
            prompt: "a tree".to_string(),
            state: VideoTaskState::Running,
            output_path: None,
            error: None,
            usage: None,
        });
        let usage = VideoUsage { billed_seconds: 5.0, billed_tokens: Some(1234), estimated_cost_usd: Some(0.05) };
        pool.set_usage("task-5", usage.clone());
        let record = pool.get("task-5").unwrap();
        assert_eq!(record.usage.unwrap().billed_tokens, Some(1234));
    }
    /// Verifies get_video_task returns an error for unknown task IDs.
    #[tokio::test]
    async fn test_get_video_task_not_found() {
        let result = get_video_task("definitely-not-a-real-task-id").await;
        assert!(result.is_err());
    }
    /// Verifies get_video_task returns a record after insertion.
    #[tokio::test]
    async fn test_get_video_task_found() {
        let task_id = format!("unit-test-{}", Uuid::new_v4());
        {
            let mut pool = VIDEO_TASK_POOL.write().await;
            pool.insert(VideoTaskRecord {
                task_id: task_id.clone(),
                provider: VideoModelProvider::Runway,
                prompt: "unit test".to_string(),
                state: VideoTaskState::Pending,
                output_path: None,
                error: None,
                usage: None,
            });
        }
        let result = get_video_task(&task_id).await;
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.task_id, task_id);
        assert_eq!(record.provider, VideoModelProvider::Runway);
        // Cleanup
        {
            let mut pool = VIDEO_TASK_POOL.write().await;
            pool.tasks.remove(&task_id);
        }
    }
    /// Verifies get_video_task_usage returns None when no usage is recorded.
    #[tokio::test]
    async fn test_get_video_task_usage_none() {
        let task_id = format!("unit-test-usage-{}", Uuid::new_v4());
        {
            let mut pool = VIDEO_TASK_POOL.write().await;
            pool.insert(VideoTaskRecord {
                task_id: task_id.clone(),
                provider: VideoModelProvider::Seedance,
                prompt: "usage test".to_string(),
                state: VideoTaskState::Succeeded,
                output_path: Some("/tmp/x.mp4".to_string()),
                error: None,
                usage: None,
            });
        }
        let result = get_video_task_usage(&task_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        // Cleanup
        {
            let mut pool = VIDEO_TASK_POOL.write().await;
            pool.tasks.remove(&task_id);
        }
    }
    /// Verifies get_video_task_usage returns the recorded usage.
    #[tokio::test]
    async fn test_get_video_task_usage_some() {
        let task_id = format!("unit-test-usage-some-{}", Uuid::new_v4());
        {
            let mut pool = VIDEO_TASK_POOL.write().await;
            pool.insert(VideoTaskRecord {
                task_id: task_id.clone(),
                provider: VideoModelProvider::Seedance,
                prompt: "usage test".to_string(),
                state: VideoTaskState::Succeeded,
                output_path: Some("/tmp/x.mp4".to_string()),
                error: None,
                usage: Some(VideoUsage { billed_seconds: 5.0, billed_tokens: Some(999), estimated_cost_usd: Some(0.05) }),
            });
        }
        let result = get_video_task_usage(&task_id).await;
        assert!(result.is_ok());
        let usage = result.unwrap().unwrap();
        assert_eq!(usage.billed_tokens, Some(999));
        // Cleanup
        {
            let mut pool = VIDEO_TASK_POOL.write().await;
            pool.tasks.remove(&task_id);
        }
    }
    /// Verifies get_media_token_count is readable.
    #[test]
    fn test_get_media_token_count_readable() {
        let count = get_media_token_count();
        assert!(count <= u64::MAX);
    }
}
