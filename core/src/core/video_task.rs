//! Video task definitions and atomic operations.
use crate::HippoxResult;
use crate::HippoxStringResult;
use crate::base64_decode_to_file;
use crate::download_to_file;
use langhub::video::VideoUsage;
use langhub::video::{VideoLLMOptions, VideoLLMResult, VideoModelProvider, VideoTask, VideoTaskStatus};
use langhub::{VideoLLMClient, VideoLLMConfig};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tracing::{debug, info, warn};
use uuid::Uuid;
/// Status of a single video generation task (public-facing, serializable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoTaskState {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Cancelled,
}
impl VideoTaskState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, VideoTaskState::Succeeded | VideoTaskState::Failed | VideoTaskState::Cancelled)
    }
    pub fn from_langhub(s: &VideoTaskStatus) -> Self {
        match s {
            VideoTaskStatus::Pending => VideoTaskState::Pending,
            VideoTaskStatus::Processing => VideoTaskState::Processing,
            VideoTaskStatus::Succeeded => VideoTaskState::Succeeded,
            VideoTaskStatus::Failed => VideoTaskState::Failed,
            VideoTaskStatus::Cancelled => VideoTaskState::Cancelled,
        }
    }
}
/// A serializable snapshot of a video generation task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoTaskInfo {
    /// Hippox-internal task id (UUID generated at submit time).
    pub task_id: String,
    /// Provider name as a plain string, e.g. "Seedance" / "Kling".
    pub provider: String,
    /// Original prompt.
    pub prompt: String,
    /// Current state.
    pub state: VideoTaskState,
    /// Human-readable status message.
    pub message: String,
    /// Progress 0-100 (may stay at 0).
    pub progress: u8,
    /// Third-party provider task id, used for polling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_task_id: Option<String>,
    /// Remote download URL of the produced video.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    /// Duration in seconds, if the provider returned it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f32>,
    /// Resolution string, if the provider returned it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// Usage reported by the provider, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<VideoUsage>,
    /// Error message, present when `state == Failed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Local path(s) after a successful `download_video_task`.
    #[serde(default)]
    pub local_paths: Vec<String>,
    /// Unix milliseconds.
    pub created_at: u64,
    /// Unix milliseconds.
    pub updated_at: u64,
}
impl VideoTaskInfo {
    pub fn new(provider: String, prompt: String) -> Self {
        let now = now_millis();
        Self {
            task_id: Uuid::new_v4().to_string(),
            provider,
            prompt,
            state: VideoTaskState::Pending,
            message: "Pending".to_string(),
            progress: 0,
            provider_task_id: None,
            download_url: None,
            duration_seconds: None,
            resolution: None,
            usage: None,
            error: None,
            local_paths: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
    fn touch(&mut self) {
        self.updated_at = now_millis();
    }
    fn set_failed(&mut self, err: String) {
        self.state = VideoTaskState::Failed;
        self.message = "Failed".to_string();
        self.error = Some(err);
        self.touch();
    }
}
fn now_millis() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}
/// Build a `VideoLLMConfig` for the given provider.
pub(crate) fn build_video_config(provider: VideoModelProvider, api_key: String, base_url: Option<String>) -> VideoLLMConfig {
    let mut config = VideoLLMConfig::new();
    match provider {
        VideoModelProvider::Seedance => {
            config = config.seedance(api_key);
            if let Some(base) = base_url {
                config.seedance_base_url = Some(base);
            }
        }
        VideoModelProvider::Wan => {
            config = config.wan(api_key);
            if let Some(base) = base_url {
                config.wan_base_url = Some(base);
            }
        }
        VideoModelProvider::Kling => {
            config = config.kling(api_key.clone(), api_key);
            if let Some(base) = base_url {
                config.kling_base_url = Some(base);
            }
        }
        VideoModelProvider::Veo => {
            config = config.veo(api_key);
            if let Some(base) = base_url {
                config.veo_base_url = Some(base);
            }
        }
        VideoModelProvider::Runway => {
            config = config.runway(api_key);
            if let Some(base) = base_url {
                config.runway_base_url = Some(base);
            }
        }
        VideoModelProvider::MiniMaxH3 => {
            config = config.minimax_h3(api_key.clone(), api_key);
            if let Some(base) = base_url {
                config.minimax_h3_base_url = Some(base);
            }
        }
        VideoModelProvider::HappyHorse => {
            config = config.happyhorse(api_key);
            if let Some(base) = base_url {
                config.happyhorse_base_url = Some(base);
            }
        }
        VideoModelProvider::Ltx => {
            config = config.ltx(api_key);
            if let Some(base) = base_url {
                config.ltx_base_url = Some(base);
            }
        }
        VideoModelProvider::GrokImagine => {
            config = config.grok(api_key);
            if let Some(base) = base_url {
                config.grok_base_url = Some(base);
            }
        }
        VideoModelProvider::Pruna => {
            config = config.pruna(api_key);
            if let Some(base) = base_url {
                config.pruna_base_url = Some(base);
            }
        }
        VideoModelProvider::GeminiOmniFlash => {
            config = config.gemini(api_key);
            if let Some(base) = base_url {
                config.gemini_base_url = Some(base);
            }
        }
    }
    config
}
/// Parse a frontend provider string to `VideoModelProvider`.
pub fn parse_video_provider(name: &str) -> Result<VideoModelProvider, String> {
    match name.to_lowercase().as_str() {
        "seedance" => Ok(VideoModelProvider::Seedance),
        "wan" => Ok(VideoModelProvider::Wan),
        "kling" => Ok(VideoModelProvider::Kling),
        "veo" => Ok(VideoModelProvider::Veo),
        "runway" => Ok(VideoModelProvider::Runway),
        "minimax_h3" | "minimaxh3" => Ok(VideoModelProvider::MiniMaxH3),
        "happyhorse" => Ok(VideoModelProvider::HappyHorse),
        "ltx" => Ok(VideoModelProvider::Ltx),
        "grok" | "grok_imagine" => Ok(VideoModelProvider::GrokImagine),
        "pruna" => Ok(VideoModelProvider::Pruna),
        "gemini" | "gemini_omni_flash" => Ok(VideoModelProvider::GeminiOmniFlash),
        other => Err(format!("Unknown video provider: {}", other)),
    }
}
/// Submit a video generation task and return immediately.
///
/// `model` - Optional model id override. When `None`, the provider's
/// configured default model is used.
pub async fn submit_video_task_info(
    provider: VideoModelProvider,
    api_key: String,
    prompt: String,
    options: Option<VideoLLMOptions>,
    base_url: Option<String>,
    model: Option<String>,
) -> HippoxResult<VideoTaskInfo> {
    let provider_name = format!("{:?}", provider);
    let mut info = VideoTaskInfo::new(provider_name.clone(), prompt.clone());
    info!(target: "hippox::media", "submit_video_task_info - provider={}, task_id={}, model={:?}", provider_name, info.task_id, model);
    let config = build_video_config(provider, api_key, base_url);
    let client = match VideoLLMClient::new_with_config(provider, &config) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Failed to create video client: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            return HippoxResult::ok(info);
        }
    };
    let opts = options.unwrap_or_default();
    match client.submit_task(&prompt, opts, model.as_deref()).await {
        Ok(task) => {
            apply_video_task_to_info(&mut info, &task);
            info.touch();
            HippoxResult::ok(info)
        }
        Err(e) => {
            let msg = format!("Video submit failed: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            HippoxResult::ok(info)
        }
    }
}
/// Poll the provider once for the current state of a video task.
pub async fn poll_video_task_info(
    provider: VideoModelProvider,
    api_key: String,
    provider_task_id: String,
    base_url: Option<String>,
    // Carried-over fields from the caller's persisted info.
    task_id: String,
    prompt: String,
    created_at: u64,
) -> HippoxResult<VideoTaskInfo> {
    let provider_name = format!("{:?}", provider);
    let mut info = VideoTaskInfo {
        task_id,
        provider: provider_name.clone(),
        prompt,
        state: VideoTaskState::Pending,
        message: "Polling".to_string(),
        progress: 0,
        provider_task_id: Some(provider_task_id.clone()),
        download_url: None,
        duration_seconds: None,
        resolution: None,
        usage: None,
        error: None,
        local_paths: Vec::new(),
        created_at,
        updated_at: now_millis(),
    };
    let config = build_video_config(provider, api_key, base_url);
    let client = match VideoLLMClient::new_with_config(provider, &config) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Failed to create video client: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            return HippoxResult::ok(info);
        }
    };
    match client.poll_task(&provider_task_id).await {
        Ok(task) => {
            apply_video_task_to_info(&mut info, &task);
            info.touch();
            HippoxResult::ok(info)
        }
        Err(e) => {
            let msg = format!("Video poll failed: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            HippoxResult::ok(info)
        }
    }
}
/// Ask the provider to cancel a video task (best effort).
pub async fn cancel_video_task_info(
    provider: VideoModelProvider,
    _api_key: String,
    _provider_task_id: Option<String>,
    _base_url: Option<String>,
    task_id: String,
    prompt: String,
    created_at: u64,
) -> HippoxResult<VideoTaskInfo> {
    let provider_name = format!("{:?}", provider);
    let mut info = VideoTaskInfo {
        task_id,
        provider: provider_name,
        prompt,
        state: VideoTaskState::Cancelled,
        message: "Cancelled".to_string(),
        progress: 0,
        provider_task_id: None,
        download_url: None,
        duration_seconds: None,
        resolution: None,
        usage: None,
        error: None,
        local_paths: Vec::new(),
        created_at,
        updated_at: now_millis(),
    };
    info.provider_task_id = _provider_task_id;
    HippoxResult::ok(info)
}
/// Download the produced video to `output_path / output_filename`.
pub async fn download_video_task(
    download_url: String,
    output_path: String,
    output_filename: Option<String>,
    task_id: String,
    provider: String,
    prompt: String,
    created_at: u64,
) -> HippoxResult<VideoTaskInfo> {
    let mut info = VideoTaskInfo {
        task_id: task_id.clone(),
        provider,
        prompt,
        state: VideoTaskState::Succeeded,
        message: "Downloading".to_string(),
        progress: 80,
        provider_task_id: None,
        download_url: Some(download_url.clone()),
        duration_seconds: None,
        resolution: None,
        usage: None,
        error: None,
        local_paths: Vec::new(),
        created_at,
        updated_at: now_millis(),
    };
    let filename = output_filename.unwrap_or_else(|| format!("{}.mp4", task_id));
    let full_path = std::path::Path::new(&output_path).join(&filename);
    if let Some(parent) = full_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            let msg = format!("Failed to create output directory: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            return HippoxResult::ok(info);
        }
    }
    if let Err(e) = download_to_file(&download_url, &full_path).await {
        let msg = format!("Failed to download video from {}: {}", download_url, e);
        warn!("{}", msg);
        info.set_failed(msg.clone());
        return HippoxResult::ok(info);
    }
    info.local_paths = vec![full_path.to_string_lossy().to_string()];
    info.progress = 100;
    info.message = "Downloaded".to_string();
    info.touch();
    HippoxResult::ok(info)
}
/// Internal helper: copy langhub task status / result / error into `info`.
fn apply_video_task_to_info(info: &mut VideoTaskInfo, task: &VideoTask) {
    info.state = VideoTaskState::from_langhub(&task.status);
    info.provider_task_id = Some(task.task_id.clone());
    info.message = match &task.status {
        VideoTaskStatus::Pending => "Pending".to_string(),
        VideoTaskStatus::Processing => "Processing".to_string(),
        VideoTaskStatus::Succeeded => "Succeeded".to_string(),
        VideoTaskStatus::Failed => "Failed".to_string(),
        VideoTaskStatus::Cancelled => "Cancelled".to_string(),
    };
    info.progress = match &task.status {
        VideoTaskStatus::Pending => 10,
        VideoTaskStatus::Processing => 50,
        VideoTaskStatus::Succeeded => 90,
        VideoTaskStatus::Failed | VideoTaskStatus::Cancelled => 100,
    };
    if let Some(result) = &task.result {
        apply_video_result_to_info(info, result);
    }
    if let Some(err) = &task.error {
        info.error = Some(err.clone());
    }
}
/// Internal helper: copy langhub result (url / duration / resolution / usage).
fn apply_video_result_to_info(info: &mut VideoTaskInfo, result: &VideoLLMResult) {
    if info.download_url.is_none() {
        info.download_url = result.video_url.clone();
    }
    if info.duration_seconds.is_none() {
        info.duration_seconds = result.duration_seconds;
    }
    if info.resolution.is_none() {
        info.resolution = result.resolution.clone();
    }
    if info.usage.is_none() {
        info.usage = result.extract_usage();
    }
}
// Legacy entry point (kept for backward compatibility).
pub(crate) async fn run_video_task(
    provider: VideoModelProvider,
    api_key: String,
    prompt: String,
    options: Option<VideoLLMOptions>,
    base_url: Option<String>,
    output_filename: Option<String>,
    output_path: String,
    model: Option<String>,
) -> HippoxStringResult {
    let submitted = submit_video_task_info(provider, api_key.clone(), prompt.clone(), options.clone(), base_url.clone(), model.clone()).await;
    let mut info = match submitted.data {
        Some(i) => i,
        None => return HippoxResult::system_error(submitted.error.unwrap_or_else(|| "Video submit failed".to_string())),
    };
    // Synchronous provider: already has a download_url.
    if info.state == VideoTaskState::Succeeded && info.download_url.is_some() {
        let dl = download_video_task(
            info.download_url.clone().unwrap(),
            output_path,
            output_filename,
            info.task_id.clone(),
            info.provider.clone(),
            info.prompt.clone(),
            info.created_at,
        )
        .await;
        return match dl.data {
            Some(dl_info) if !dl_info.local_paths.is_empty() => HippoxResult::ok(dl_info.local_paths[0].clone()),
            Some(dl_info) => HippoxResult::system_error(dl_info.error.unwrap_or_else(|| "Video download failed".to_string())),
            None => HippoxResult::system_error(dl.error.unwrap_or_else(|| "Video download failed".to_string())),
        };
    }
    // Async provider: poll until terminal.
    let provider_task_id = match info.provider_task_id.clone() {
        Some(id) => id,
        None => return HippoxResult::system_error("Video provider returned no task id".to_string()),
    };
    for _ in 0..180 {
        let polled = poll_video_task_info(
            provider,
            api_key.clone(),
            provider_task_id.clone(),
            base_url.clone(),
            info.task_id.clone(),
            info.prompt.clone(),
            info.created_at,
        )
        .await;
        match polled.data {
            Some(p) => {
                info = p;
                if info.state == VideoTaskState::Succeeded {
                    let url = match info.download_url.clone() {
                        Some(u) => u,
                        None => return HippoxResult::system_error("Video succeeded but no download_url".to_string()),
                    };
                    let dl = download_video_task(
                        url,
                        output_path,
                        output_filename,
                        info.task_id.clone(),
                        info.provider.clone(),
                        info.prompt.clone(),
                        info.created_at,
                    )
                    .await;
                    return match dl.data {
                        Some(dl_info) if !dl_info.local_paths.is_empty() => HippoxResult::ok(dl_info.local_paths[0].clone()),
                        Some(dl_info) => HippoxResult::system_error(dl_info.error.unwrap_or_else(|| "Video download failed".to_string())),
                        None => HippoxResult::system_error(dl.error.unwrap_or_else(|| "Video download failed".to_string())),
                    };
                }
                if info.state.is_terminal() {
                    return HippoxResult::system_error(info.error.unwrap_or_else(|| "Video task failed".to_string()));
                }
            }
            None => {
                return HippoxResult::system_error(polled.error.unwrap_or_else(|| "Video poll failed".to_string()));
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
    HippoxResult::system_error("Video task polling timeout".to_string())
}
