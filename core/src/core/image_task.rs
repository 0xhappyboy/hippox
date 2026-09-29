//! Image task definitions and dedicated task pool.
//!
//! # Entry point
//! The public entry is `Hippox::submit_image_task` in `hippox.rs`, which calls
//! `run_image_task` in this module.
//!
//! # Design
//! - Uses a dedicated `IMAGE_TASK_POOL` (independent from the general `TASK_POOL`).
//! - Directly calls `ImageLLMClient` from `langhub`.
//! - Downloads the produced image(s) to `output_path` (or `output_path/output_filename`
//!   when a custom filename is provided; for multiple images, an index suffix is added).
//! - Records real usage (if the provider returns it) into the task record.
//!   Image usage has no token field, so it does NOT contribute to `MEDIA_TOKEN_COUNT`.
use crate::HippoxResult;
use crate::HippoxStringResult;
use crate::base64_decode_to_file;
use crate::download_to_file;
use langhub::image::{ImageLLMOptions, ImageModelProvider, ImageUsage};
use langhub::types::Result as LangHubResult;
use langhub::{ImageLLMClient, ImageLLMConfig};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
/// Status of a single image generation task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageTaskState {
    /// Task has been created but not started yet.
    Pending,
    /// Task is currently calling the LLM API.
    Running,
    /// Task completed successfully and the image(s) were saved to disk.
    Succeeded,
    /// Task failed with an error message.
    Failed,
}
/// A single image generation task record.
#[derive(Debug, Clone)]
pub struct ImageTaskRecord {
    /// Unique task ID.
    pub task_id: String,
    /// Provider used for this task.
    pub provider: ImageModelProvider,
    /// Original prompt.
    pub prompt: String,
    /// Current task state.
    pub state: ImageTaskState,
    /// Local paths where the produced image(s) were saved (on success).
    pub output_paths: Vec<String>,
    /// Error message (on failure).
    pub error: Option<String>,
    /// Real usage reported by the provider (if any).
    pub usage: Option<ImageUsage>,
}
/// Dedicated task pool for image generation tasks.
#[derive(Debug, Default)]
pub struct ImageTaskPool {
    tasks: HashMap<String, ImageTaskRecord>,
}
impl ImageTaskPool {
    /// Creates a new empty image task pool.
    pub fn new() -> Self {
        Self { tasks: HashMap::new() }
    }
    /// Inserts a new task record and returns its ID.
    pub fn insert(&mut self, record: ImageTaskRecord) -> String {
        let id = record.task_id.clone();
        self.tasks.insert(id.clone(), record);
        id
    }
    /// Updates the state of an existing task.
    pub fn set_state(&mut self, task_id: &str, state: ImageTaskState) {
        if let Some(record) = self.tasks.get_mut(task_id) {
            record.state = state;
        }
    }
    /// Marks a task as succeeded and records the output paths.
    pub fn set_succeeded(&mut self, task_id: &str, output_paths: Vec<String>) {
        if let Some(record) = self.tasks.get_mut(task_id) {
            record.state = ImageTaskState::Succeeded;
            record.output_paths = output_paths;
            record.error = None;
        }
    }
    /// Marks a task as failed and records the error message.
    pub fn set_failed(&mut self, task_id: &str, error: String) {
        if let Some(record) = self.tasks.get_mut(task_id) {
            record.state = ImageTaskState::Failed;
            record.error = Some(error);
        }
    }
    /// Records usage for a task.
    pub fn set_usage(&mut self, task_id: &str, usage: ImageUsage) {
        if let Some(record) = self.tasks.get_mut(task_id) {
            record.usage = Some(usage);
        }
    }
    /// Gets a clone of a task record by ID.
    pub fn get(&self, task_id: &str) -> Option<ImageTaskRecord> {
        self.tasks.get(task_id).cloned()
    }
}
/// Global dedicated image task pool.
pub static IMAGE_TASK_POOL: Lazy<Arc<RwLock<ImageTaskPool>>> = Lazy::new(|| Arc::new(RwLock::new(ImageTaskPool::new())));
/// Get the current state of an image task.
pub async fn get_image_task(task_id: &str) -> HippoxResult<ImageTaskRecord> {
    let pool = IMAGE_TASK_POOL.read().await;
    match pool.get(task_id) {
        Some(record) => HippoxResult::ok(record),
        None => HippoxResult::system_error(format!("Image task not found: {}", task_id)),
    }
}
/// Get the usage recorded for a specific image task.
pub async fn get_image_task_usage(task_id: &str) -> HippoxResult<Option<ImageUsage>> {
    let pool = IMAGE_TASK_POOL.read().await;
    match pool.get(task_id) {
        Some(record) => HippoxResult::ok(record.usage),
        None => HippoxResult::system_error(format!("Image task not found: {}", task_id)),
    }
}
/// Internal implementation of an image generation task.
pub(crate) async fn run_image_task(
    provider: ImageModelProvider,
    api_key: String,
    prompt: String,
    options: Option<ImageLLMOptions>,
    base_url: Option<String>,
    output_filename: Option<String>,
    output_path: String,
) -> HippoxStringResult {
    let task_id = Uuid::new_v4().to_string();
    info!("Submitting image task {}: provider={:?}, prompt_len={}", task_id, provider, prompt.len());
    {
        let mut pool = IMAGE_TASK_POOL.write().await;
        pool.insert(ImageTaskRecord {
            task_id: task_id.clone(),
            provider,
            prompt: prompt.clone(),
            state: ImageTaskState::Pending,
            output_paths: Vec::new(),
            error: None,
            usage: None,
        });
    }
    {
        let mut pool = IMAGE_TASK_POOL.write().await;
        pool.set_state(&task_id, ImageTaskState::Running);
    }
    let mut config = ImageLLMConfig::new();
    config = match provider {
        ImageModelProvider::Seedream => {
            let mut c = config.seedream(api_key.clone());
            if let Some(base) = &base_url {
                c.seedream_base_url = Some(base.clone());
            }
            c
        }
        ImageModelProvider::WanImage => {
            let mut c = config.wan_image(api_key.clone());
            if let Some(base) = &base_url {
                c.wan_image_base_url = Some(base.clone());
            }
            c
        }
        ImageModelProvider::StabilityImage => {
            let mut c = config.stability(api_key.clone());
            if let Some(base) = &base_url {
                c.stability_base_url = Some(base.clone());
            }
            c
        }
        ImageModelProvider::Flux => {
            let mut c = config.flux(api_key.clone());
            if let Some(base) = &base_url {
                c.flux_base_url = Some(base.clone());
            }
            c
        }
        ImageModelProvider::Imagen => {
            let mut c = config.imagen(api_key.clone());
            if let Some(base) = &base_url {
                c.imagen_base_url = Some(base.clone());
            }
            c
        }
        ImageModelProvider::DallE => {
            let mut c = config.dalle(api_key.clone());
            if let Some(base) = &base_url {
                c.dalle_base_url = Some(base.clone());
            }
            c
        }
    };
    let client = match ImageLLMClient::new_with_config(provider, &config) {
        Ok(c) => c,
        Err(e) => {
            let err_msg = format!("Failed to create image client: {}", e);
            warn!("{}", err_msg);
            let mut pool = IMAGE_TASK_POOL.write().await;
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
            let err_msg = format!("Image generation failed: {}", e);
            warn!("{}", err_msg);
            let mut pool = IMAGE_TASK_POOL.write().await;
            pool.set_failed(&task_id, err_msg.clone());
            return HippoxResult::system_error(err_msg);
        }
    };
    // Record real usage into the task record.
    // `ImageUsage` has no token field, so it does NOT feed `MEDIA_TOKEN_COUNT`.
    if let Some(usage) = result.extract_usage() {
        let mut pool = IMAGE_TASK_POOL.write().await;
        pool.set_usage(&task_id, usage);
    }
    if let Err(e) = std::fs::create_dir_all(&output_path) {
        let err_msg = format!("Failed to create output directory: {}", e);
        warn!("{}", err_msg);
        let mut pool = IMAGE_TASK_POOL.write().await;
        pool.set_failed(&task_id, err_msg.clone());
        return HippoxResult::system_error(err_msg);
    }
    let mut saved_paths: Vec<String> = Vec::new();
    let total = result.image_urls.len();
    for (idx, url) in result.image_urls.iter().enumerate() {
        let filename = match &output_filename {
            Some(name) => {
                if total > 1 {
                    // Append index suffix for multi-image output.
                    let path = std::path::Path::new(name);
                    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("png");
                    format!("{}_{}.{}", stem, idx, ext)
                } else {
                    name.clone()
                }
            }
            None => format!("{}_{}.png", task_id, idx),
        };
        let full_path = std::path::Path::new(&output_path).join(&filename);
        if let Err(e) = download_to_file(url, &full_path).await {
            let err_msg = format!("Failed to download image from {}: {}", url, e);
            warn!("{}", err_msg);
            let mut pool = IMAGE_TASK_POOL.write().await;
            pool.set_failed(&task_id, err_msg.clone());
            return HippoxResult::system_error(err_msg);
        }
        saved_paths.push(full_path.to_string_lossy().to_string());
    }
    if saved_paths.is_empty() {
        if let Some(b64_list) = &result.image_base64 {
            let total_b64 = b64_list.len();
            for (idx, b64) in b64_list.iter().enumerate() {
                let filename = match &output_filename {
                    Some(name) => {
                        if total_b64 > 1 {
                            let path = std::path::Path::new(name);
                            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
                            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("png");
                            format!("{}_{}.{}", stem, idx, ext)
                        } else {
                            name.clone()
                        }
                    }
                    None => format!("{}_{}.png", task_id, idx),
                };
                let full_path = std::path::Path::new(&output_path).join(&filename);
                if let Err(e) = base64_decode_to_file(b64, &full_path) {
                    let err_msg = format!("Failed to decode base64 image: {}", e);
                    warn!("{}", err_msg);
                    let mut pool = IMAGE_TASK_POOL.write().await;
                    pool.set_failed(&task_id, err_msg.clone());
                    return HippoxResult::system_error(err_msg);
                }
                saved_paths.push(full_path.to_string_lossy().to_string());
            }
        }
    }
    if saved_paths.is_empty() {
        let err_msg = "Image generation returned no image URL or base64 payload".to_string();
        warn!("{}", err_msg);
        let mut pool = IMAGE_TASK_POOL.write().await;
        pool.set_failed(&task_id, err_msg.clone());
        return HippoxResult::system_error(err_msg);
    }
    info!("Image task {} saved {} image(s) to {}", task_id, saved_paths.len(), output_path);
    let first_path = saved_paths[0].clone();
    {
        let mut pool = IMAGE_TASK_POOL.write().await;
        pool.set_succeeded(&task_id, saved_paths);
    }
    HippoxResult::ok(first_path)
}
#[cfg(test)]
mod image_task_tests {
    use super::*;
    /// Verifies that a new ImageTaskPool is empty.
    #[test]
    fn test_image_task_pool_new_is_empty() {
        let pool = ImageTaskPool::new();
        assert!(pool.get("nonexistent").is_none());
    }
    /// Verifies insert / get round-trip on ImageTaskPool.
    #[test]
    fn test_image_task_pool_insert_and_get() {
        let mut pool = ImageTaskPool::new();
        let record = ImageTaskRecord {
            task_id: "img-1".to_string(),
            provider: ImageModelProvider::Seedream,
            prompt: "a red apple".to_string(),
            state: ImageTaskState::Pending,
            output_paths: Vec::new(),
            error: None,
            usage: None,
        };
        let id = pool.insert(record);
        assert_eq!(id, "img-1");
        let fetched = pool.get("img-1").unwrap();
        assert_eq!(fetched.task_id, "img-1");
        assert_eq!(fetched.provider, ImageModelProvider::Seedream);
        assert_eq!(fetched.state, ImageTaskState::Pending);
    }
    /// Verifies set_state updates the task state.
    #[test]
    fn test_image_task_pool_set_state() {
        let mut pool = ImageTaskPool::new();
        pool.insert(ImageTaskRecord {
            task_id: "img-2".to_string(),
            provider: ImageModelProvider::WanImage,
            prompt: "a blue sky".to_string(),
            state: ImageTaskState::Pending,
            output_paths: Vec::new(),
            error: None,
            usage: None,
        });
        pool.set_state("img-2", ImageTaskState::Running);
        assert_eq!(pool.get("img-2").unwrap().state, ImageTaskState::Running);
    }
    /// Verifies set_succeeded records the output paths and clears error.
    #[test]
    fn test_image_task_pool_set_succeeded() {
        let mut pool = ImageTaskPool::new();
        pool.insert(ImageTaskRecord {
            task_id: "img-3".to_string(),
            provider: ImageModelProvider::Flux,
            prompt: "a green tree".to_string(),
            state: ImageTaskState::Running,
            output_paths: Vec::new(),
            error: Some("previous error".to_string()),
            usage: None,
        });
        let paths = vec!["/tmp/a.png".to_string(), "/tmp/b.png".to_string()];
        pool.set_succeeded("img-3", paths.clone());
        let record = pool.get("img-3").unwrap();
        assert_eq!(record.state, ImageTaskState::Succeeded);
        assert_eq!(record.output_paths, paths);
        assert!(record.error.is_none());
    }
    /// Verifies set_failed records the error message.
    #[test]
    fn test_image_task_pool_set_failed() {
        let mut pool = ImageTaskPool::new();
        pool.insert(ImageTaskRecord {
            task_id: "img-4".to_string(),
            provider: ImageModelProvider::Imagen,
            prompt: "a yellow sun".to_string(),
            state: ImageTaskState::Running,
            output_paths: Vec::new(),
            error: None,
            usage: None,
        });
        pool.set_failed("img-4", "boom".to_string());
        let record = pool.get("img-4").unwrap();
        assert_eq!(record.state, ImageTaskState::Failed);
        assert_eq!(record.error, Some("boom".to_string()));
    }
    /// Verifies set_state on a missing task is a no-op.
    #[test]
    fn test_image_task_pool_set_state_missing() {
        let mut pool = ImageTaskPool::new();
        pool.set_state("nonexistent", ImageTaskState::Running);
        assert!(pool.get("nonexistent").is_none());
    }
    /// Verifies set_usage records the usage on the task.
    #[test]
    fn test_image_task_pool_set_usage() {
        let mut pool = ImageTaskPool::new();
        pool.insert(ImageTaskRecord {
            task_id: "img-5".to_string(),
            provider: ImageModelProvider::Seedream,
            prompt: "a flower".to_string(),
            state: ImageTaskState::Running,
            output_paths: Vec::new(),
            error: None,
            usage: None,
        });
        let usage = ImageUsage { billed_images: 2, billed_megapixels: Some(1.5), estimated_cost_usd: Some(0.04) };
        pool.set_usage("img-5", usage.clone());
        let record = pool.get("img-5").unwrap();
        assert_eq!(record.usage.unwrap().billed_images, 2);
    }
    /// Verifies get_image_task returns an error for unknown task IDs.
    #[tokio::test]
    async fn test_get_image_task_not_found() {
        let result = get_image_task("definitely-not-a-real-task-id").await;
        assert!(result.is_err());
    }
    /// Verifies get_image_task returns a record after insertion.
    #[tokio::test]
    async fn test_get_image_task_found() {
        let task_id = format!("unit-test-{}", Uuid::new_v4());
        {
            let mut pool = IMAGE_TASK_POOL.write().await;
            pool.insert(ImageTaskRecord {
                task_id: task_id.clone(),
                provider: ImageModelProvider::DallE,
                prompt: "unit test".to_string(),
                state: ImageTaskState::Pending,
                output_paths: Vec::new(),
                error: None,
                usage: None,
            });
        }
        let result = get_image_task(&task_id).await;
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.task_id, task_id);
        assert_eq!(record.provider, ImageModelProvider::DallE);
        // Cleanup
        {
            let mut pool = IMAGE_TASK_POOL.write().await;
            pool.tasks.remove(&task_id);
        }
    }
    /// Verifies get_image_task_usage returns None when no usage is recorded.
    #[tokio::test]
    async fn test_get_image_task_usage_none() {
        let task_id = format!("unit-test-usage-{}", Uuid::new_v4());
        {
            let mut pool = IMAGE_TASK_POOL.write().await;
            pool.insert(ImageTaskRecord {
                task_id: task_id.clone(),
                provider: ImageModelProvider::Seedream,
                prompt: "usage test".to_string(),
                state: ImageTaskState::Succeeded,
                output_paths: vec!["/tmp/x.png".to_string()],
                error: None,
                usage: None,
            });
        }
        let result = get_image_task_usage(&task_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        // Cleanup
        {
            let mut pool = IMAGE_TASK_POOL.write().await;
            pool.tasks.remove(&task_id);
        }
    }
    /// Verifies get_image_task_usage returns the recorded usage.
    #[tokio::test]
    async fn test_get_image_task_usage_some() {
        let task_id = format!("unit-test-usage-some-{}", Uuid::new_v4());
        {
            let mut pool = IMAGE_TASK_POOL.write().await;
            pool.insert(ImageTaskRecord {
                task_id: task_id.clone(),
                provider: ImageModelProvider::Seedream,
                prompt: "usage test".to_string(),
                state: ImageTaskState::Succeeded,
                output_paths: vec!["/tmp/x.png".to_string()],
                error: None,
                usage: Some(ImageUsage { billed_images: 1, billed_megapixels: Some(1.0), estimated_cost_usd: Some(0.02) }),
            });
        }
        let result = get_image_task_usage(&task_id).await;
        assert!(result.is_ok());
        let usage = result.unwrap().unwrap();
        assert_eq!(usage.billed_images, 1);
        // Cleanup
        {
            let mut pool = IMAGE_TASK_POOL.write().await;
            pool.tasks.remove(&task_id);
        }
    }
}
