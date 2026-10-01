//! Image task definitions and atomic operations.
use crate::HippoxResult;
use crate::HippoxStringResult;
use crate::base64_decode_to_file;
use crate::download_to_file;
use langhub::image::ImageUsage;
use langhub::image::{ImageLLMOptions, ImageLLMResult, ImageModelProvider, ImageTask, ImageTaskStatus};
use langhub::{ImageLLMClient, ImageLLMConfig};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tracing::{info, warn};
use uuid::Uuid;
/// Status of a single image generation task (public-facing, serializable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageTaskState {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Cancelled,
}
impl ImageTaskState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, ImageTaskState::Succeeded | ImageTaskState::Failed | ImageTaskState::Cancelled)
    }
    pub fn from_langhub(s: &ImageTaskStatus) -> Self {
        match s {
            ImageTaskStatus::Pending => ImageTaskState::Pending,
            ImageTaskStatus::Processing => ImageTaskState::Processing,
            ImageTaskStatus::Succeeded => ImageTaskState::Succeeded,
            ImageTaskStatus::Failed => ImageTaskState::Failed,
            ImageTaskStatus::Cancelled => ImageTaskState::Cancelled,
        }
    }
}
/// A serializable snapshot of an image generation task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageTaskInfo {
    pub task_id: String,
    pub provider: String,
    pub prompt: String,
    pub state: ImageTaskState,
    pub message: String,
    pub progress: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_task_id: Option<String>,
    /// Multiple download URLs (images are usually multi-output).
    #[serde(default)]
    pub download_urls: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<ImageUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default)]
    pub local_paths: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}
impl ImageTaskInfo {
    pub fn new(provider: String, prompt: String) -> Self {
        let now = now_millis();
        Self {
            task_id: Uuid::new_v4().to_string(),
            provider,
            prompt,
            state: ImageTaskState::Pending,
            message: "Pending".to_string(),
            progress: 0,
            provider_task_id: None,
            download_urls: Vec::new(),
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
        self.state = ImageTaskState::Failed;
        self.message = "Failed".to_string();
        self.error = Some(err);
        self.touch();
    }
}
fn now_millis() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}
/// Build an `ImageLLMConfig` for the given provider.
pub(crate) fn build_image_config(provider: ImageModelProvider, api_key: String, base_url: Option<String>) -> ImageLLMConfig {
    let mut config = ImageLLMConfig::new();
    match provider {
        ImageModelProvider::Seedream => {
            config = config.seedream(api_key);
            if let Some(base) = base_url {
                config.seedream_base_url = Some(base);
            }
        }
        ImageModelProvider::WanImage => {
            config = config.wan_image(api_key);
            if let Some(base) = base_url {
                config.wan_image_base_url = Some(base);
            }
        }
        ImageModelProvider::StabilityImage => {
            config = config.stability(api_key);
            if let Some(base) = base_url {
                config.stability_base_url = Some(base);
            }
        }
        ImageModelProvider::Flux => {
            config = config.flux(api_key);
            if let Some(base) = base_url {
                config.flux_base_url = Some(base);
            }
        }
        ImageModelProvider::Imagen => {
            config = config.imagen(api_key);
            if let Some(base) = base_url {
                config.imagen_base_url = Some(base);
            }
        }
        ImageModelProvider::DallE => {
            config = config.dalle(api_key);
            if let Some(base) = base_url {
                config.dalle_base_url = Some(base);
            }
        }
    }
    config
}
/// Parse a frontend provider string to `ImageModelProvider`.
pub fn parse_image_provider(name: &str) -> Result<ImageModelProvider, String> {
    match name.to_lowercase().as_str() {
        "seedream" => Ok(ImageModelProvider::Seedream),
        "wan_image" | "wanimage" | "wan" => Ok(ImageModelProvider::WanImage),
        "stability" | "stability_image" => Ok(ImageModelProvider::StabilityImage),
        "flux" => Ok(ImageModelProvider::Flux),
        "imagen" => Ok(ImageModelProvider::Imagen),
        "dalle" | "dall_e" | "dall-e" => Ok(ImageModelProvider::DallE),
        other => Err(format!("Unknown image provider: {}", other)),
    }
}
/// Submit an image generation task and return immediately.
///
/// `model` - Optional model id override. When `None`, the provider's
/// configured default model is used.
pub async fn submit_image_task_info(
    provider: ImageModelProvider,
    api_key: String,
    prompt: String,
    options: Option<ImageLLMOptions>,
    base_url: Option<String>,
    model: Option<String>,
) -> HippoxResult<ImageTaskInfo> {
    let provider_name = format!("{:?}", provider);
    let mut info = ImageTaskInfo::new(provider_name.clone(), prompt.clone());
    info!(target: "hippox::media", "submit_image_task_info - provider={}, task_id={}, model={:?}", provider_name, info.task_id, model);
    let config = build_image_config(provider, api_key, base_url);
    let client = match ImageLLMClient::new_with_config(provider, &config) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Failed to create image client: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            return HippoxResult::ok(info);
        }
    };
    let opts = options.unwrap_or_default();
    match client.submit_task(&prompt, opts, model.as_deref()).await {
        Ok(task) => {
            apply_image_task_to_info(&mut info, &task);
            info.touch();
            HippoxResult::ok(info)
        }
        Err(e) => {
            let msg = format!("Image submit failed: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            HippoxResult::ok(info)
        }
    }
}
pub async fn poll_image_task_info(
    provider: ImageModelProvider,
    api_key: String,
    provider_task_id: String,
    base_url: Option<String>,
    task_id: String,
    prompt: String,
    created_at: u64,
) -> HippoxResult<ImageTaskInfo> {
    let provider_name = format!("{:?}", provider);
    let mut info = ImageTaskInfo {
        task_id,
        provider: provider_name.clone(),
        prompt,
        state: ImageTaskState::Pending,
        message: "Polling".to_string(),
        progress: 0,
        provider_task_id: Some(provider_task_id.clone()),
        download_urls: Vec::new(),
        resolution: None,
        usage: None,
        error: None,
        local_paths: Vec::new(),
        created_at,
        updated_at: now_millis(),
    };
    let config = build_image_config(provider, api_key, base_url);
    let client = match ImageLLMClient::new_with_config(provider, &config) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Failed to create image client: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            return HippoxResult::ok(info);
        }
    };
    match client.poll_task(&provider_task_id).await {
        Ok(task) => {
            apply_image_task_to_info(&mut info, &task);
            info.touch();
            HippoxResult::ok(info)
        }
        Err(e) => {
            let msg = format!("Image poll failed: {}", e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            HippoxResult::ok(info)
        }
    }
}
pub async fn cancel_image_task_info(
    provider: ImageModelProvider,
    _api_key: String,
    provider_task_id: Option<String>,
    _base_url: Option<String>,
    task_id: String,
    prompt: String,
    created_at: u64,
) -> HippoxResult<ImageTaskInfo> {
    let provider_name = format!("{:?}", provider);
    let info = ImageTaskInfo {
        task_id,
        provider: provider_name,
        prompt,
        state: ImageTaskState::Cancelled,
        message: "Cancelled".to_string(),
        progress: 0,
        provider_task_id,
        download_urls: Vec::new(),
        resolution: None,
        usage: None,
        error: None,
        local_paths: Vec::new(),
        created_at,
        updated_at: now_millis(),
    };
    HippoxResult::ok(info)
}
pub async fn download_image_task(
    download_urls: Vec<String>,
    output_path: String,
    output_filename: Option<String>,
    task_id: String,
    provider: String,
    prompt: String,
    created_at: u64,
) -> HippoxResult<ImageTaskInfo> {
    let mut info = ImageTaskInfo {
        task_id: task_id.clone(),
        provider,
        prompt,
        state: ImageTaskState::Succeeded,
        message: "Downloading".to_string(),
        progress: 80,
        provider_task_id: None,
        download_urls: download_urls.clone(),
        resolution: None,
        usage: None,
        error: None,
        local_paths: Vec::new(),
        created_at,
        updated_at: now_millis(),
    };
    if let Err(e) = std::fs::create_dir_all(&output_path) {
        let msg = format!("Failed to create output directory: {}", e);
        warn!("{}", msg);
        info.set_failed(msg.clone());
        return HippoxResult::ok(info);
    }
    let total = download_urls.len();
    let mut saved: Vec<String> = Vec::new();
    for (idx, url) in download_urls.iter().enumerate() {
        let filename = match &output_filename {
            Some(name) if total > 1 => {
                let path = std::path::Path::new(name);
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("png");
                format!("{}_{}.{}", stem, idx, ext)
            }
            Some(name) => name.clone(),
            None => format!("{}_{}.png", task_id, idx),
        };
        let full_path = std::path::Path::new(&output_path).join(&filename);
        if let Err(e) = download_to_file(url, &full_path).await {
            let msg = format!("Failed to download image from {}: {}", url, e);
            warn!("{}", msg);
            info.set_failed(msg.clone());
            return HippoxResult::ok(info);
        }
        saved.push(full_path.to_string_lossy().to_string());
    }
    if saved.is_empty() {
        let msg = "Image download received no URL".to_string();
        info.set_failed(msg);
        return HippoxResult::ok(info);
    }
    info.local_paths = saved;
    info.progress = 100;
    info.message = "Downloaded".to_string();
    info.touch();
    HippoxResult::ok(info)
}
fn apply_image_task_to_info(info: &mut ImageTaskInfo, task: &ImageTask) {
    info.state = ImageTaskState::from_langhub(&task.status);
    info.provider_task_id = Some(task.task_id.clone());
    info.message = match &task.status {
        ImageTaskStatus::Pending => "Pending".to_string(),
        ImageTaskStatus::Processing => "Processing".to_string(),
        ImageTaskStatus::Succeeded => "Succeeded".to_string(),
        ImageTaskStatus::Failed => "Failed".to_string(),
        ImageTaskStatus::Cancelled => "Cancelled".to_string(),
    };
    info.progress = match &task.status {
        ImageTaskStatus::Pending => 10,
        ImageTaskStatus::Processing => 50,
        ImageTaskStatus::Succeeded => 90,
        ImageTaskStatus::Failed | ImageTaskStatus::Cancelled => 100,
    };
    if let Some(result) = &task.result {
        apply_image_result_to_info(info, result);
    }
    if let Some(err) = &task.error {
        info.error = Some(err.clone());
    }
}
fn apply_image_result_to_info(info: &mut ImageTaskInfo, result: &ImageLLMResult) {
    if info.download_urls.is_empty() {
        info.download_urls = result.image_urls.clone();
    }
    if info.resolution.is_none() {
        info.resolution = result.resolution.clone();
    }
    if info.usage.is_none() {
        info.usage = result.extract_usage();
    }
}
// Legacy entry point (kept for backward compatibility).
pub(crate) async fn run_image_task(
    provider: ImageModelProvider,
    api_key: String,
    prompt: String,
    options: Option<ImageLLMOptions>,
    base_url: Option<String>,
    output_filename: Option<String>,
    output_path: String,
    model: Option<String>,
) -> HippoxStringResult {
    let submitted = submit_image_task_info(provider, api_key.clone(), prompt.clone(), options.clone(), base_url.clone(), model.clone()).await;
    let mut info = match submitted.data {
        Some(i) => i,
        None => return HippoxResult::system_error(submitted.error.unwrap_or_else(|| "Image submit failed".to_string())),
    };
    // Sync provider: already Succeeded with URLs.
    if info.state == ImageTaskState::Succeeded && !info.download_urls.is_empty() {
        let dl = download_image_task(
            info.download_urls.clone(),
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
            Some(dl_info) => HippoxResult::system_error(dl_info.error.unwrap_or_else(|| "Image download failed".to_string())),
            None => HippoxResult::system_error(dl.error.unwrap_or_else(|| "Image download failed".to_string())),
        };
    }
    // Async provider: poll until terminal.
    let provider_task_id = match info.provider_task_id.clone() {
        Some(id) => id,
        None => {
            return run_image_task_sync_fallback(provider, api_key, prompt, options, base_url, output_filename, output_path, model).await;
        }
    };
    for _ in 0..180 {
        let polled = poll_image_task_info(
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
                if info.state == ImageTaskState::Succeeded {
                    if info.download_urls.is_empty() {
                        return HippoxResult::system_error("Image succeeded but no download_urls".to_string());
                    }
                    let dl = download_image_task(
                        info.download_urls.clone(),
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
                        Some(dl_info) => HippoxResult::system_error(dl_info.error.unwrap_or_else(|| "Image download failed".to_string())),
                        None => HippoxResult::system_error(dl.error.unwrap_or_else(|| "Image download failed".to_string())),
                    };
                }
                if info.state.is_terminal() {
                    return HippoxResult::system_error(info.error.unwrap_or_else(|| "Image task failed".to_string()));
                }
            }
            None => {
                return HippoxResult::system_error(polled.error.unwrap_or_else(|| "Image poll failed".to_string()));
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
    HippoxResult::system_error("Image task polling timeout".to_string())
}
async fn run_image_task_sync_fallback(
    provider: ImageModelProvider,
    api_key: String,
    prompt: String,
    options: Option<ImageLLMOptions>,
    base_url: Option<String>,
    output_filename: Option<String>,
    output_path: String,
    model: Option<String>,
) -> HippoxStringResult {
    let config = build_image_config(provider, api_key, base_url);
    let client = match ImageLLMClient::new_with_config(provider, &config) {
        Ok(c) => c,
        Err(e) => return HippoxResult::system_error(format!("Failed to create image client: {}", e)),
    };
    let result = match options {
        Some(opts) => client.generate_with_options(&prompt, opts, model.as_deref()).await,
        None => client.generate(&prompt, model.as_deref()).await,
    };
    let result = match result {
        Ok(r) => r,
        Err(e) => return HippoxResult::system_error(format!("Image generation failed: {}", e)),
    };
    if let Err(e) = std::fs::create_dir_all(&output_path) {
        return HippoxResult::system_error(format!("Failed to create output directory: {}", e));
    }
    let task_id = Uuid::new_v4().to_string();
    let mut saved: Vec<String> = Vec::new();
    let total = result.image_urls.len();
    for (idx, url) in result.image_urls.iter().enumerate() {
        let filename = match &output_filename {
            Some(name) if total > 1 => {
                let p = std::path::Path::new(name);
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
                let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("png");
                format!("{}_{}.{}", stem, idx, ext)
            }
            Some(name) => name.clone(),
            None => format!("{}_{}.png", task_id, idx),
        };
        let full_path = std::path::Path::new(&output_path).join(&filename);
        if let Err(e) = download_to_file(url, &full_path).await {
            return HippoxResult::system_error(format!("Failed to download image: {}", e));
        }
        saved.push(full_path.to_string_lossy().to_string());
    }
    if saved.is_empty() {
        if let Some(b64_list) = &result.image_base64 {
            let total_b64 = b64_list.len();
            for (idx, b64) in b64_list.iter().enumerate() {
                let filename = match &output_filename {
                    Some(name) if total_b64 > 1 => {
                        let p = std::path::Path::new(name);
                        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
                        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("png");
                        format!("{}_{}.{}", stem, idx, ext)
                    }
                    Some(name) => name.clone(),
                    None => format!("{}_{}.png", task_id, idx),
                };
                let full_path = std::path::Path::new(&output_path).join(&filename);
                if let Err(e) = base64_decode_to_file(b64, &full_path) {
                    return HippoxResult::system_error(format!("Failed to decode base64 image: {}", e));
                }
                saved.push(full_path.to_string_lossy().to_string());
            }
        }
    }
    if saved.is_empty() {
        return HippoxResult::system_error("Image generation returned no image URL or base64 payload".to_string());
    }
    info!("Image task fallback saved {} image(s) to {}", saved.len(), output_path);
    HippoxResult::ok(saved[0].clone())
}
