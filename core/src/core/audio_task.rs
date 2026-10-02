//! Audio task definitions and atomic operations.
use crate::HippoxResult;
use crate::HippoxStringResult;
use crate::base64_decode_to_file;
use crate::download_to_file;
use langhub::audio::{AudioLLMOptions, AudioLLMResult, AudioModelProvider, AudioTask, AudioTaskStatus, AudioUsage};
use langhub::{AudioLLMClient, AudioLLMConfig};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tracing::{info, warn};
use uuid::Uuid;
/// Status of a single audio generation task (public-facing, serializable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioTaskState {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Cancelled,
}
impl AudioTaskState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, AudioTaskState::Succeeded | AudioTaskState::Failed | AudioTaskState::Cancelled)
    }
    pub fn from_langhub(s: &AudioTaskStatus) -> Self {
        match s {
            AudioTaskStatus::Pending => AudioTaskState::Pending,
            AudioTaskStatus::Processing => AudioTaskState::Processing,
            AudioTaskStatus::Succeeded => AudioTaskState::Succeeded,
            AudioTaskStatus::Failed => AudioTaskState::Failed,
            AudioTaskStatus::Cancelled => AudioTaskState::Cancelled,
        }
    }
}
/// A serializable snapshot of an audio generation task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTaskInfo {
    pub task_id: String,
    pub provider: String,
    pub prompt: String,
    pub state: AudioTaskState,
    pub message: String,
    pub progress: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<AudioUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default)]
    pub local_paths: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}
impl AudioTaskInfo {
    pub fn new(provider: String, prompt: String) -> Self {
        let now = now_millis();
        Self {
            task_id: Uuid::new_v4().to_string(),
            provider,
            prompt,
            state: AudioTaskState::Pending,
            message: "Pending".to_string(),
            progress: 0,
            provider_task_id: None,
            download_url: None,
            format: None,
            duration_seconds: None,
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
        self.state = AudioTaskState::Failed;
        self.message = "Failed".to_string();
        self.error = Some(err);
        self.touch();
    }
}
fn now_millis() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}
/// Build an `AudioLLMConfig` for the given provider.
pub fn build_audio_config(provider: AudioModelProvider, api_key: String, base_url: Option<String>) -> AudioLLMConfig {
    let mut config = AudioLLMConfig::new();
    match provider {
        AudioModelProvider::QwenTts => {
            config = config.qwen_tts(api_key);
            if let Some(base) = base_url {
                config.qwen_tts_base_url = Some(base);
            }
        }
        AudioModelProvider::SeedAudio => {
            config = config.seed_audio(api_key);
            if let Some(base) = base_url {
                config.seed_audio_base_url = Some(base);
            }
        }
        AudioModelProvider::StepAudio => {
            config = config.step_audio(api_key);
            if let Some(base) = base_url {
                config.step_audio_base_url = Some(base);
            }
        }
        AudioModelProvider::GeminiTts => {
            config = config.gemini_tts(api_key);
            if let Some(base) = base_url {
                config.gemini_tts_base_url = Some(base);
            }
        }
        AudioModelProvider::ElevenLabs => {
            config = config.elevenlabs(api_key);
            if let Some(base) = base_url {
                config.elevenlabs_base_url = Some(base);
            }
        }
        AudioModelProvider::Lyria => {
            config = config.lyria(api_key);
            if let Some(base) = base_url {
                config.lyria_base_url = Some(base);
            }
        }
        AudioModelProvider::Suno => {
            config = config.suno(api_key);
            if let Some(base) = base_url {
                config.suno_base_url = Some(base);
            }
        }
        AudioModelProvider::StableAudio => {
            config = config.stable_audio(api_key);
            if let Some(base) = base_url {
                config.stable_audio_base_url = Some(base);
            }
        }
    }
    config
}
/// Parse a frontend provider string to `AudioModelProvider`.
pub fn parse_audio_provider(name: &str) -> Result<AudioModelProvider, String> {
    match name.to_lowercase().as_str() {
        "qwen_tts" | "qwentts" => Ok(AudioModelProvider::QwenTts),
        "seed_audio" | "seedaudio" => Ok(AudioModelProvider::SeedAudio),
        "step_audio" | "stepaudio" => Ok(AudioModelProvider::StepAudio),
        "gemini_tts" | "geminitts" => Ok(AudioModelProvider::GeminiTts),
        "elevenlabs" => Ok(AudioModelProvider::ElevenLabs),
        "lyria" => Ok(AudioModelProvider::Lyria),
        "suno" => Ok(AudioModelProvider::Suno),
        "stable_audio" | "stableaudio" => Ok(AudioModelProvider::StableAudio),
        other => Err(format!("Unknown audio provider: {}", other)),
    }
}
// Atomic operations
pub async fn submit_audio_task_info(
    provider: AudioModelProvider,
    api_key: String,
    prompt: String,
    options: Option<AudioLLMOptions>,
    base_url: Option<String>,
    model: Option<String>,
) -> HippoxResult<AudioTaskInfo> {
    let provider_name = format!("{:?}", provider);
    let mut info = AudioTaskInfo::new(provider_name.clone(), prompt.clone());
    info!(target: "hippox::media", "submit_audio_task_info - provider={}, task_id={}, model={:?}", provider_name, info.task_id, model);
    let config = build_audio_config(provider, api_key, base_url);
    let client = match AudioLLMClient::new_with_config(provider, &config) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Failed to create audio client: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            return HippoxResult::ok(info);
        }
    };
    let opts = options.unwrap_or_default();
    match client.submit_task(&prompt, opts, model.as_deref()).await {
        Ok(task) => {
            apply_audio_task_to_info(&mut info, &task);
            info.touch();
            HippoxResult::ok(info)
        }
        Err(e) => {
            let msg = format!("Audio submit failed: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            HippoxResult::ok(info)
        }
    }
}
pub async fn poll_audio_task_info(
    provider: AudioModelProvider,
    api_key: String,
    provider_task_id: String,
    base_url: Option<String>,
    task_id: String,
    prompt: String,
    created_at: u64,
) -> HippoxResult<AudioTaskInfo> {
    let provider_name = format!("{:?}", provider);
    let mut info = AudioTaskInfo {
        task_id,
        provider: provider_name.clone(),
        prompt,
        state: AudioTaskState::Pending,
        message: "Polling".to_string(),
        progress: 0,
        provider_task_id: Some(provider_task_id.clone()),
        download_url: None,
        format: None,
        duration_seconds: None,
        usage: None,
        error: None,
        local_paths: Vec::new(),
        created_at,
        updated_at: now_millis(),
    };
    let config = build_audio_config(provider, api_key, base_url);
    let client = match AudioLLMClient::new_with_config(provider, &config) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Failed to create audio client: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            return HippoxResult::ok(info);
        }
    };
    match client.poll_task(&provider_task_id).await {
        Ok(task) => {
            apply_audio_task_to_info(&mut info, &task);
            info.touch();
            HippoxResult::ok(info)
        }
        Err(e) => {
            let msg = format!("Audio poll failed: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            HippoxResult::ok(info)
        }
    }
}
pub async fn cancel_audio_task_info(
    provider: AudioModelProvider,
    _api_key: String,
    provider_task_id: Option<String>,
    _base_url: Option<String>,
    task_id: String,
    prompt: String,
    created_at: u64,
) -> HippoxResult<AudioTaskInfo> {
    let provider_name = format!("{:?}", provider);
    let info = AudioTaskInfo {
        task_id,
        provider: provider_name,
        prompt,
        state: AudioTaskState::Cancelled,
        message: "Cancelled".to_string(),
        progress: 0,
        provider_task_id,
        download_url: None,
        format: None,
        duration_seconds: None,
        usage: None,
        error: None,
        local_paths: Vec::new(),
        created_at,
        updated_at: now_millis(),
    };
    HippoxResult::ok(info)
}
pub async fn download_audio_task(
    download_url: Option<String>,
    audio_base64: Option<String>,
    format: Option<String>,
    output_path: String,
    output_filename: Option<String>,
    task_id: String,
    provider: String,
    prompt: String,
    created_at: u64,
) -> HippoxResult<AudioTaskInfo> {
    let mut info = AudioTaskInfo {
        task_id: task_id.clone(),
        provider,
        prompt,
        state: AudioTaskState::Succeeded,
        message: "Downloading".to_string(),
        progress: 80,
        provider_task_id: None,
        download_url: download_url.clone(),
        format: format.clone(),
        duration_seconds: None,
        usage: None,
        error: None,
        local_paths: Vec::new(),
        created_at,
        updated_at: now_millis(),
    };
    let default_ext = format.unwrap_or_else(|| "mp3".to_string());
    let filename = output_filename.unwrap_or_else(|| format!("{}.{}", task_id, default_ext));
    let full_path = std::path::Path::new(&output_path).join(&filename);
    if let Some(parent) = full_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            let msg = format!("Failed to create output directory: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            return HippoxResult::ok(info);
        }
    }
    if let Some(url) = &download_url {
        if let Err(e) = download_to_file(url, &full_path).await {
            let msg = format!("Failed to download audio from {}: {}", url, e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            return HippoxResult::ok(info);
        }
    } else if let Some(b64) = &audio_base64 {
        if let Err(e) = base64_decode_to_file(b64, &full_path) {
            let msg = format!("Failed to decode base64 audio: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            return HippoxResult::ok(info);
        }
    } else {
        let msg = "Audio download received neither URL nor base64".to_string();
        info.set_failed(msg);
        return HippoxResult::ok(info);
    }
    info.local_paths = vec![full_path.to_string_lossy().to_string()];
    info.progress = 100;
    info.message = "Downloaded".to_string();
    info.touch();
    HippoxResult::ok(info)
}
fn apply_audio_task_to_info(info: &mut AudioTaskInfo, task: &AudioTask) {
    info.state = AudioTaskState::from_langhub(&task.status);
    info.provider_task_id = Some(task.task_id.clone());
    info.message = match &task.status {
        AudioTaskStatus::Pending => "Pending".to_string(),
        AudioTaskStatus::Processing => "Processing".to_string(),
        AudioTaskStatus::Succeeded => "Succeeded".to_string(),
        AudioTaskStatus::Failed => "Failed".to_string(),
        AudioTaskStatus::Cancelled => "Cancelled".to_string(),
    };
    info.progress = match &task.status {
        AudioTaskStatus::Pending => 10,
        AudioTaskStatus::Processing => 50,
        AudioTaskStatus::Succeeded => 90,
        AudioTaskStatus::Failed | AudioTaskStatus::Cancelled => 100,
    };
    if let Some(result) = &task.result {
        apply_audio_result_to_info(info, result);
    }
    if let Some(err) = &task.error {
        info.error = Some(err.clone());
    }
}
fn apply_audio_result_to_info(info: &mut AudioTaskInfo, result: &AudioLLMResult) {
    if info.download_url.is_none() {
        info.download_url = result.audio_url.clone();
    }
    if info.format.is_none() {
        info.format = result.format.clone();
    }
    if info.duration_seconds.is_none() {
        info.duration_seconds = result.duration_seconds;
    }
    if info.usage.is_none() {
        info.usage = result.extract_usage();
    }
}
// Legacy entry point (kept for backward compatibility).
pub(crate) async fn run_audio_task(
    provider: AudioModelProvider,
    api_key: String,
    prompt: String,
    options: Option<AudioLLMOptions>,
    base_url: Option<String>,
    output_filename: Option<String>,
    output_path: String,
    model: Option<String>,
) -> HippoxStringResult {
    let submitted = submit_audio_task_info(provider, api_key.clone(), prompt.clone(), options.clone(), base_url.clone(), model.clone()).await;
    let mut info = match submitted.data {
        Some(i) => i,
        None => return HippoxResult::system_error(submitted.error.unwrap_or_else(|| "Audio submit failed".to_string())),
    };
    // Sync provider: already Succeeded.
    if info.state == AudioTaskState::Succeeded {
        let dl = download_audio_task(
            info.download_url.clone(),
            None,
            info.format.clone(),
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
            Some(dl_info) => HippoxResult::system_error(dl_info.error.unwrap_or_else(|| "Audio download failed".to_string())),
            None => HippoxResult::system_error(dl.error.unwrap_or_else(|| "Audio download failed".to_string())),
        };
    }
    // Async provider: poll until terminal.
    let provider_task_id = match info.provider_task_id.clone() {
        Some(id) => id,
        None => {
            return run_audio_task_sync_fallback(provider, api_key, prompt, options, base_url, output_filename, output_path, model).await;
        }
    };
    for _ in 0..180 {
        let polled = poll_audio_task_info(
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
                if info.state == AudioTaskState::Succeeded {
                    let dl = download_audio_task(
                        info.download_url.clone(),
                        None,
                        info.format.clone(),
                        output_path.clone(),
                        output_filename.clone(),
                        info.task_id.clone(),
                        info.provider.clone(),
                        info.prompt.clone(),
                        info.created_at,
                    )
                    .await;
                    return match dl.data {
                        Some(dl_info) if !dl_info.local_paths.is_empty() => HippoxResult::ok(dl_info.local_paths[0].clone()),
                        Some(dl_info) => HippoxResult::system_error(dl_info.error.unwrap_or_else(|| "Audio download failed".to_string())),
                        None => HippoxResult::system_error(dl.error.unwrap_or_else(|| "Audio download failed".to_string())),
                    };
                }
                if info.state.is_terminal() {
                    return HippoxResult::system_error(info.error.unwrap_or_else(|| "Audio task failed".to_string()));
                }
            }
            None => {
                return HippoxResult::system_error(polled.error.unwrap_or_else(|| "Audio poll failed".to_string()));
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
    HippoxResult::system_error("Audio task polling timeout".to_string())
}
/// Fallback path for providers that don't return a task id.
async fn run_audio_task_sync_fallback(
    provider: AudioModelProvider,
    api_key: String,
    prompt: String,
    options: Option<AudioLLMOptions>,
    base_url: Option<String>,
    output_filename: Option<String>,
    output_path: String,
    model: Option<String>,
) -> HippoxStringResult {
    let config = build_audio_config(provider, api_key, base_url);
    let client = match AudioLLMClient::new_with_config(provider, &config) {
        Ok(c) => c,
        Err(e) => return HippoxResult::system_error(format!("Failed to create audio client: {}", e)),
    };
    let result = match options {
        Some(opts) => client.generate_with_options(&prompt, opts, model.as_deref()).await,
        None => client.generate(&prompt, model.as_deref()).await,
    };
    let result = match result {
        Ok(r) => r,
        Err(e) => return HippoxResult::system_error(format!("Audio generation failed: {}", e)),
    };
    let task_id = Uuid::new_v4().to_string();
    let default_ext = result.format.clone().unwrap_or_else(|| "mp3".to_string());
    let filename = output_filename.unwrap_or_else(|| format!("{}.{}", task_id, default_ext));
    let full_path = std::path::Path::new(&output_path).join(&filename);
    if let Some(parent) = full_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return HippoxResult::system_error(format!("Failed to create output directory: {}", e));
        }
    }
    if let Some(url) = &result.audio_url {
        if let Err(e) = download_to_file(url, &full_path).await {
            return HippoxResult::system_error(format!("Failed to download audio: {}", e));
        }
    } else if let Some(b64) = &result.audio_base64 {
        if let Err(e) = base64_decode_to_file(b64, &full_path) {
            return HippoxResult::system_error(format!("Failed to decode base64 audio: {}", e));
        }
    } else {
        return HippoxResult::system_error("Audio generation returned no audio URL or base64 payload".to_string());
    }
    info!("Audio task fallback saved to {}", full_path.to_string_lossy());
    HippoxResult::ok(full_path.to_string_lossy().to_string())
}
