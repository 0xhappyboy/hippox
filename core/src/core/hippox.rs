use crate::core::tasks::NaturalLanguageTask;
use crate::driver_scheduler::DriverScheduler;
use crate::prompts::{build_driver_md_prompt, generate_drivers_registry};
use crate::tasks::{self, ExecutableTask, TaskStatus};
use crate::workflow::{WorkflowCallback, WorkflowExecutionResult, WorkflowExecutor, WorkflowMode};
use crate::{
    HippoxBatchResult, HippoxBoolResult, HippoxConfig, HippoxResult, HippoxStringResult, HippoxVoidResult, IdentityInformation, IntentAnalysisResult,
    Pipeline, SystemPipeline, WorkflowExecResult, get_config, i18n, needs_format_conversion, t, update_config,
};
use hippox_drivers::{DriverCallback, DriverCategory, Executor, get_all_drivers, list_drivers_names};
use langhub::audio::{AudioLLMOptions, AudioModelProvider};
use langhub::chat::ChatModelProvider;
use langhub::image::{ImageLLMOptions, ImageModelProvider};
use langhub::types::ChatMessage;
use langhub::video::{VideoLLMOptions, VideoModelProvider};
use langhub::{AudioLLMClient, AudioLLMConfig, ImageLLMClient, ImageLLMConfig, LLMClient, VideoLLMClient, VideoLLMConfig};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tracing::info;
/// Global input token count for the entire process
pub static INPUT_TOKEN_COUNT: AtomicU64 = AtomicU64::new(0);
/// Global output token count for the entire process
pub static OUTPUT_TOKEN_COUNT: AtomicU64 = AtomicU64::new(0);
/// Core engine for Hippox.
#[derive(Clone)]
pub struct Hippox {
    scheduler: DriverScheduler,
    executor: Executor,
    is_first_message: Arc<AtomicBool>,
    /// Optional image generation client (attached via `with_image_client`).
    image_client: Option<Arc<ImageLLMClient>>,
    /// Image provider used when `image_client` is attached.
    image_provider: Option<ImageModelProvider>,
    /// Optional video generation client (attached via `with_video_client`).
    video_client: Option<Arc<VideoLLMClient>>,
    /// Video provider used when `video_client` is attached.
    video_provider: Option<VideoModelProvider>,
    /// Optional audio generation client (attached via `with_audio_client`).
    audio_client: Option<Arc<AudioLLMClient>>,
    /// Audio provider used when `audio_client` is attached.
    audio_provider: Option<AudioModelProvider>,
}
impl Hippox {
    /// Create a new Hippox core instance with default ReAct workflow mode
    pub async fn new(
        provider: ChatModelProvider,
        api_key: Option<String>,
        extra_keys: Option<HashMap<String, String>>,
        config: Option<HippoxConfig>,
    ) -> anyhow::Result<Self> {
        Self::with_workflow_mode(provider, api_key, extra_keys, config).await
    }
    /// Create a new Hippox core instance with specified workflow mode
    pub async fn with_workflow_mode(
        provider: ChatModelProvider,
        api_key: Option<String>,
        extra_keys: Option<HashMap<String, String>>,
        config: Option<HippoxConfig>,
    ) -> anyhow::Result<Self> {
        // init config
        update_config(|global| *global = config.unwrap_or_default())?;
        // set i18n
        let config = get_config();
        i18n::set_language(&config.lang);
        // init llm
        let llm = LLMClient::new_with_key(provider, api_key, extra_keys)?;
        // init llm scheduler
        let scheduler = DriverScheduler::new(llm);
        let executor = Executor::new();
        Ok(Self {
            scheduler,
            executor,
            is_first_message: Arc::new(AtomicBool::new(false)),
            image_client: None,
            image_provider: None,
            video_client: None,
            video_provider: None,
            audio_client: None,
            audio_provider: None,
        })
    }
    /// Create a `Hippox` instance without a real LLM scheduler.
    pub async fn without_llm(config: Option<HippoxConfig>) -> anyhow::Result<Self> {
        update_config(|global| *global = config.unwrap_or_default())?;
        let config = get_config();
        i18n::set_language(&config.lang);
        let llm = LLMClient::new_with_key(ChatModelProvider::OpenAI, Some(String::new()), None)?;
        let scheduler = DriverScheduler::new(llm);
        let executor = Executor::new();
        Ok(Self {
            scheduler,
            executor,
            is_first_message: Arc::new(AtomicBool::new(false)),
            image_client: None,
            image_provider: None,
            video_client: None,
            video_provider: None,
            audio_client: None,
            audio_provider: None,
        })
    }
    /// Attach (or replace) the image generation client.
    pub fn with_image_client(mut self, client: ImageLLMClient, provider: ImageModelProvider) -> Self {
        self.image_client = Some(Arc::new(client));
        self.image_provider = Some(provider);
        self
    }
    /// Attach (or replace) the video generation client.
    pub fn with_video_client(mut self, client: VideoLLMClient, provider: VideoModelProvider) -> Self {
        self.video_client = Some(Arc::new(client));
        self.video_provider = Some(provider);
        self
    }
    /// Attach (or replace) the audio generation client.
    pub fn with_audio_client(mut self, client: AudioLLMClient, provider: AudioModelProvider) -> Self {
        self.audio_client = Some(Arc::new(client));
        self.audio_provider = Some(provider);
        self
    }
    /// Returns the embedded image client when present.
    pub fn image_client(&self) -> Option<(&ImageLLMClient, ImageModelProvider)> {
        match (&self.image_client, self.image_provider) {
            (Some(client), Some(provider)) => Some((client.as_ref(), provider)),
            _ => None,
        }
    }
    /// Returns the embedded video client when present.
    pub fn video_client(&self) -> Option<(&VideoLLMClient, VideoModelProvider)> {
        match (&self.video_client, self.video_provider) {
            (Some(client), Some(provider)) => Some((client.as_ref(), provider)),
            _ => None,
        }
    }
    /// Returns the embedded audio client when present.
    pub fn audio_client(&self) -> Option<(&AudioLLMClient, AudioModelProvider)> {
        match (&self.audio_client, self.audio_provider) {
            (Some(client), Some(provider)) => Some((client.as_ref(), provider)),
            _ => None,
        }
    }
    /// Image modality instantiation — mirrors LLMClient::new_with_key
    /// Create an image client using an optional API key
    ///
    /// # Arguments
    /// * `provider`   - The image model provider to use.
    /// * `api_key`    - Optional API key for the provider.
    /// * `extra_keys` - Optional extra keys (e.g. `base_url`) for providers
    ///                  that need them.
    ///
    /// # Example
    /// ```ignore
    /// let client = Hippox::new_llm_image_with_key(
    ///     ImageModelProvider::Seedream,
    ///     Some("ark-key".to_string()),
    ///     None,
    /// )?;
    /// let result = client.generate("a cat sitting on a windowsill").await?;
    /// ```
    pub fn new_llm_image_with_key(
        provider: ImageModelProvider,
        api_key: Option<String>,
        extra_keys: Option<HashMap<String, String>>,
    ) -> langhub::types::Result<ImageLLMClient> {
        ImageLLMClient::new_with_key(provider, api_key, extra_keys)
    }
    /// Create an image client from an `ImageLLMConfig`
    ///
    /// # Arguments
    /// * `provider` - The image model provider to use.
    /// * `config`   - Configuration containing API keys and credentials.
    ///
    /// # Example
    /// ```ignore
    /// let config = ImageLLMConfig::new()
    ///     .seedream("ark-key".to_string())
    ///     .dalle("openai-key".to_string());
    /// let client = Hippox::new_llm_image_with_config(ImageModelProvider::Seedream, &config)?;
    /// ```
    pub fn new_llm_image_with_config(provider: ImageModelProvider, config: &ImageLLMConfig) -> langhub::types::Result<ImageLLMClient> {
        ImageLLMClient::new_with_config(provider, config)
    }
    /// Video modality instantiation — mirrors LLMClient::new_with_key
    /// Create a video client using an optional API key
    ///
    /// # Arguments
    /// * `provider`   - The video model provider to use.
    /// * `api_key`    - Optional API key for the provider.
    /// * `extra_keys` - Optional extra keys (e.g. `base_url`, `secret_key`,
    ///                  `group_id`) for providers that need them.
    ///
    /// # Example
    /// ```ignore
    /// let client = Hippox::new_llm_video_with_key(
    ///     VideoModelProvider::Seedance,
    ///     Some("ark-key".to_string()),
    ///     None,
    /// )?;
    /// let result = client.generate("a cat walking on the beach").await?;
    /// ```
    pub fn new_llm_video_with_key(
        provider: VideoModelProvider,
        api_key: Option<String>,
        extra_keys: Option<HashMap<String, String>>,
    ) -> langhub::types::Result<VideoLLMClient> {
        VideoLLMClient::new_with_key(provider, api_key, extra_keys)
    }
    /// Create a video client from a `VideoLLMConfig`
    ///
    /// # Arguments
    /// * `provider` - The video model provider to use.
    /// * `config`   - Configuration containing API keys and credentials.
    ///
    /// # Example
    /// ```ignore
    /// let config = VideoLLMConfig::new()
    ///     .seedance("ark-key".to_string())
    ///     .wan("dashscope-key".to_string());
    /// let client = Hippox::new_llm_video_with_config(VideoModelProvider::Seedance, &config)?;
    /// ```
    pub fn new_llm_video_with_config(provider: VideoModelProvider, config: &VideoLLMConfig) -> langhub::types::Result<VideoLLMClient> {
        VideoLLMClient::new_with_config(provider, config)
    }
    /// Audio modality instantiation — mirrors LLMClient::new_with_key
    /// Create an audio client using an optional API key
    ///
    /// # Arguments
    /// * `provider`   - The audio model provider to use.
    /// * `api_key`    - Optional API key for the provider.
    /// * `extra_keys` - Optional extra keys (e.g. `base_url`) for providers
    ///                  that need them.
    ///
    /// # Example
    /// ```ignore
    /// let client = Hippox::new_llm_audio_with_key(
    ///     AudioModelProvider::QwenTts,
    ///     Some("dashscope-key".to_string()),
    ///     None,
    /// )?;
    /// let result = client.generate("Hello, world!").await?;
    /// ```
    pub fn new_llm_audio_with_key(
        provider: AudioModelProvider,
        api_key: Option<String>,
        extra_keys: Option<HashMap<String, String>>,
    ) -> langhub::types::Result<AudioLLMClient> {
        AudioLLMClient::new_with_key(provider, api_key, extra_keys)
    }
    /// Create an audio client from an `AudioLLMConfig`
    ///
    /// # Arguments
    /// * `provider` - The audio model provider to use.
    /// * `config`   - Configuration containing API keys and credentials.
    ///
    /// # Example
    /// ```ignore
    /// let config = AudioLLMConfig::new()
    ///     .qwen_tts("dashscope-key".to_string())
    ///     .elevenlabs("elevenlabs-key".to_string());
    /// let client = Hippox::new_llm_audio_with_config(AudioModelProvider::QwenTts, &config)?;
    /// ```
    pub fn new_llm_audio_with_config(provider: AudioModelProvider, config: &AudioLLMConfig) -> langhub::types::Result<AudioLLMClient> {
        AudioLLMClient::new_with_config(provider, config)
    }
    /// Notify LLM about updated drivers registry
    pub fn refresh_llm_driver_registry(&self) -> HippoxVoidResult {
        self.is_first_message.store(false, Ordering::SeqCst);
        HippoxResult::ok(())
    }
    /// Notify LLM about updated instances registry
    pub fn refresh_llm_instances(&self) -> HippoxVoidResult {
        self.is_first_message.store(false, Ordering::SeqCst);
        HippoxResult::ok(())
    }
    /// Get current drivers registry as JSON string
    pub fn get_drivers_registry(&self) -> HippoxStringResult {
        HippoxResult::ok(generate_drivers_registry())
    }
    /// Get identity information
    pub fn get_identity(&self) -> HippoxResult<IdentityInformation> {
        HippoxResult::ok(self.get_config().identity_information)
    }
    /// Update identity information with a closure
    pub fn update_identity<F>(&self, f: F) -> HippoxVoidResult
    where
        F: FnOnce(&mut IdentityInformation),
    {
        match self.update_config(|config| {
            f(&mut config.identity_information);
        }) {
            Ok(_) => HippoxResult::ok(()),
            Err(e) => HippoxResult::system_error(e.to_string()),
        }
    }
    /// Set identity information directly
    pub fn set_identity(&self, identity: IdentityInformation) -> HippoxVoidResult {
        match self.update_config(|config| {
            config.identity_information = identity;
        }) {
            Ok(_) => HippoxResult::ok(()),
            Err(e) => HippoxResult::system_error(e.to_string()),
        }
    }
    /// Submit a natural language task and return task ID immediately
    ///
    /// # Arguments
    /// * `input` - Natural language input from the user
    /// * `_session_id` - Optional session ID (unused in core, for compatibility)
    /// * `_callback` - Optional callback for workflow execution progress
    ///
    /// # Returns
    /// The task ID as a string wrapped in HippoxResult
    pub fn submit(
        &self,
        input: &str,
        workflow_mode: WorkflowMode,
        workflow_callback: Option<Arc<dyn WorkflowCallback>>,
        driver_callback: Option<Arc<dyn DriverCallback>>,
        disabled_drivers: Option<Vec<&str>>,
    ) -> HippoxStringResult {
        let workflow_executor = WorkflowExecutor::new(workflow_mode);
        let executable = Arc::new(NaturalLanguageTask::new(
            input.to_string(),
            workflow_executor,
            self.scheduler.clone(),
            workflow_callback,
            driver_callback,
            disabled_drivers,
        ));
        let result = futures::executor::block_on(tasks::create_task_with_executable("natural_language".to_string(), input.to_string(), executable));
        if result.is_ok() {
            let task_id = result.unwrap();
            info!("Created natural language task: {} with input: {}", task_id, input);
            HippoxResult::ok(task_id)
        } else {
            let error = result.error.unwrap_or_else(|| "Unknown error".to_string());
            tracing::error!("Failed to create task: {}", error);
            HippoxResult::system_error(format!("Failed to create task: {}", error))
        }
    }
    /// Submit multiple natural language tasks in batch and return task IDs immediately
    ///
    /// # Arguments
    /// * `inputs` - Vector of tuples (input, session_id, workflow_callback, driver_callback)
    ///
    /// # Returns
    /// Vector of task IDs in the same order as inputs wrapped in HippoxResult
    pub fn submit_batch(
        &self,
        inputs: Vec<(String, WorkflowMode, Option<String>, Option<Arc<dyn WorkflowCallback>>, Option<Arc<dyn DriverCallback>>, Option<Vec<&str>>)>,
    ) -> HippoxBatchResult {
        let task_ids: Vec<String> = inputs
            .into_iter()
            .map(|(input, workflow_mode, _session_id, workflow_callback, driver_callback, disabled_drivers)| {
                self.submit(&input, workflow_mode, workflow_callback, driver_callback, disabled_drivers).unwrap_or(String::new())
            })
            .collect();
        HippoxResult::ok(task_ids)
    }
    /// Execute multiple natural language tasks in batch and return results directly
    ///
    /// # Arguments
    /// * `inputs` - Vector of tuples (input, workflow_callback, driver_callback)
    ///
    /// # Returns
    /// Vector of results in the same order as inputs wrapped in HippoxBatchResult
    pub async fn execute_batch(
        &self,
        inputs: Vec<(String, WorkflowMode, Option<Arc<dyn WorkflowCallback>>, Option<Arc<dyn DriverCallback>>, Option<Vec<&str>>)>,
    ) -> HippoxBatchResult {
        let mut results = Vec::new();
        for (input, workflow_mode, workflow_callback, driver_callback, disabled_drivers) in inputs {
            results.push(self.execute(&input, workflow_mode, workflow_callback, driver_callback, disabled_drivers).await.unwrap_or(String::new()));
        }
        HippoxResult::ok(results)
    }
    /// Execute natural language directly without task pool, returning the result asynchronously.
    ///
    /// # Example
    /// ```
    /// # async fn example() -> anyhow::Result<()> {
    /// let result = hippox.execute("What is the weather today?", None).await?;
    /// println!("{}", result);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Compare with [`submit()`](Self::submit):
    /// - `execute()`: Blocks until completion, returns result directly
    /// - `submit()`: Returns task ID immediately, use [`wait_task()`](Self::wait_task) to get result
    pub async fn execute(
        &self,
        input: &str,
        workflow_mode: WorkflowMode,
        workflow_callback: Option<Arc<dyn WorkflowCallback>>,
        driver_callback: Option<Arc<dyn DriverCallback>>,
        disabled_drivers: Option<Vec<&str>>,
    ) -> HippoxStringResult {
        let workflow_executor = WorkflowExecutor::new(workflow_mode);
        let temp_task_id = uuid::Uuid::new_v4().to_string();
        {
            let mut pool = tasks::TASK_POOL.write().await;
            let task = tasks::Task::new("temp".to_string(), input.to_string());
            pool.tasks.insert(temp_task_id.clone(), task);
        }
        let pipeline = SystemPipeline::new();
        let disabled_drivers_owned = disabled_drivers.map(|v| v.into_iter().map(String::from).collect::<Vec<_>>());
        // Step 1: intent analysis
        let intent_result = match pipeline.intent_analysis(&self.scheduler, input, &temp_task_id).await {
            Ok(result) => result,
            Err(e) => {
                tracing::warn!("Intent analysis failed: {}, using raw input", e);
                IntentAnalysisResult { categories: vec![], clean_intent: input.to_string() }
            }
        };
        let clean_intent = &intent_result.clean_intent;
        let categories = &intent_result.categories;
        // Workflow execution
        let workflow_executor_with_id = workflow_executor.clone().with_task_id(temp_task_id.clone());
        // workflow callback
        let workflow_executor_with_callbacks =
            if let Some(cb) = workflow_callback { workflow_executor_with_id.with_workflow_callback(cb) } else { workflow_executor_with_id };
        // driver callback
        let workflow_executor_with_driver_cb =
            if let Some(cb) = driver_callback { workflow_executor_with_callbacks.with_driver_callback(cb) } else { workflow_executor_with_callbacks };
        let workflow_result = if categories.is_empty() {
            pipeline
                .workflow_execution(
                    workflow_mode,
                    &workflow_executor_with_driver_cb,
                    &self.scheduler,
                    clean_intent,
                    disabled_drivers_owned.as_deref(),
                )
                .await
        } else {
            let result = workflow_executor_with_driver_cb
                .clone()
                .execute_with_categories(&self.scheduler, clean_intent, categories, disabled_drivers_owned.as_deref())
                .await;
            let json_output = match result {
                WorkflowExecutionResult::Completed(output) => output,
                WorkflowExecutionResult::CompletedWithRaw { raw_json, .. } => raw_json,
                _ => String::new(),
            };
            WorkflowExecResult { json_output, original_input: clean_intent.to_string() }
        };
        // Step 3: format conversion
        let final_output = if needs_format_conversion(input) {
            let format_result = pipeline.response_formatting(&self.scheduler, input, &workflow_result.json_output, &temp_task_id).await;
            format_result.final_output
        } else {
            workflow_result.json_output
        };
        let (input_tokens, output_tokens) =
            tasks::get_task(&temp_task_id).await.map(|task| (task.input_token_count, task.output_token_count)).unwrap_or((0, 0));
        {
            let mut pool = tasks::TASK_POOL.write().await;
            // Remove temporary tasks from the task pool.
            pool.tasks.remove(&temp_task_id);
        }
        INPUT_TOKEN_COUNT.fetch_add(input_tokens, std::sync::atomic::Ordering::Relaxed);
        OUTPUT_TOKEN_COUNT.fetch_add(output_tokens, std::sync::atomic::Ordering::Relaxed);
        HippoxResult::ok_with_tokens(final_output, input_tokens, output_tokens)
    }
    /// Submit a video generation task and return immediately.
    ///
    /// # Arguments
    /// * `provider` - The video model provider to use.
    /// * `api_key` - API key for the provider.
    /// * `prompt` - Text prompt for video generation.
    /// * `options` - Optional generation options (duration, resolution, etc.).
    /// * `base_url` - Optional custom base URL for the provider.
    ///
    /// # Returns
    /// `VideoTaskInfo` containing the hippox task id, the provider task id,
    /// the current state, the download url (when available) and the usage.
    /// The caller is responsible for persisting this info and polling.
    pub async fn submit_video_task_info(
        &self,
        provider: VideoModelProvider,
        api_key: String,
        prompt: String,
        options: Option<VideoLLMOptions>,
        base_url: Option<String>,
    ) -> HippoxResult<crate::core::video_task::VideoTaskInfo> {
        crate::core::video_task::submit_video_task_info(provider, api_key, prompt, options, base_url).await
    }
    /// Poll the provider once for the current state of a video task.
    ///
    /// # Arguments
    /// * `provider` - The video model provider to use.
    /// * `api_key` - API key for the provider.
    /// * `provider_task_id` - Third-party task id, obtained from the previous
    ///   `submit_video_task_info` / `poll_video_task_info` call.
    /// * `base_url` - Optional custom base URL for the provider.
    /// * `task_id` - Hippox-internal task id, carried over from persistence.
    /// * `prompt` - Original prompt, carried over from persistence.
    /// * `created_at` - Original creation timestamp, carried over from persistence.
    ///
    /// # Returns
    /// `VideoTaskInfo` with the latest state / download url / usage.
    pub async fn poll_video_task_info(
        &self,
        provider: VideoModelProvider,
        api_key: String,
        provider_task_id: String,
        base_url: Option<String>,
        task_id: String,
        prompt: String,
        created_at: u64,
    ) -> HippoxResult<crate::core::video_task::VideoTaskInfo> {
        crate::core::video_task::poll_video_task_info(provider, api_key, provider_task_id, base_url, task_id, prompt, created_at).await
    }
    /// Cancel a video generation task (best effort, local-only for now).
    pub async fn cancel_video_task_info(
        &self,
        provider: VideoModelProvider,
        api_key: String,
        provider_task_id: Option<String>,
        base_url: Option<String>,
        task_id: String,
        prompt: String,
        created_at: u64,
    ) -> HippoxResult<crate::core::video_task::VideoTaskInfo> {
        crate::core::video_task::cancel_video_task_info(provider, api_key, provider_task_id, base_url, task_id, prompt, created_at).await
    }
    /// Download the produced video to `output_path / output_filename`.
    ///
    /// # Arguments
    /// * `download_url` - Remote URL returned by the previous poll.
    /// * `output_path` - Directory where the video will be saved.
    /// * `output_filename` - Optional filename; when `None` a default
    ///   `{task_id}.mp4` name is used.
    /// * `task_id` - Hippox-internal task id, carried over from persistence.
    /// * `provider` - Provider name, carried over from persistence.
    /// * `prompt` - Original prompt, carried over from persistence.
    /// * `created_at` - Original creation timestamp, carried over from persistence.
    ///
    /// # Returns
    /// `VideoTaskInfo` with `local_paths` populated on success.
    pub async fn download_video_task(
        &self,
        download_url: String,
        output_path: String,
        output_filename: Option<String>,
        task_id: String,
        provider: String,
        prompt: String,
        created_at: u64,
    ) -> HippoxResult<crate::core::video_task::VideoTaskInfo> {
        crate::core::video_task::download_video_task(download_url, output_path, output_filename, task_id, provider, prompt, created_at).await
    }
    /// Submit an image generation task and return immediately.
    ///
    /// # Arguments
    /// * `provider` - The image model provider to use.
    /// * `api_key` - API key for the provider.
    /// * `prompt` - Text prompt for image generation.
    /// * `options` - Optional generation options (n, resolution, aspect ratio, etc.).
    /// * `base_url` - Optional custom base URL for the provider.
    ///
    /// # Returns
    /// `ImageTaskInfo` containing the hippox task id, the provider task id,
    /// the current state, the download urls (when available) and the usage.
    pub async fn submit_image_task_info(
        &self,
        provider: ImageModelProvider,
        api_key: String,
        prompt: String,
        options: Option<ImageLLMOptions>,
        base_url: Option<String>,
    ) -> HippoxResult<crate::core::image_task::ImageTaskInfo> {
        crate::core::image_task::submit_image_task_info(provider, api_key, prompt, options, base_url).await
    }
    /// Poll the provider once for the current state of an image task.
    pub async fn poll_image_task_info(
        &self,
        provider: ImageModelProvider,
        api_key: String,
        provider_task_id: String,
        base_url: Option<String>,
        task_id: String,
        prompt: String,
        created_at: u64,
    ) -> HippoxResult<crate::core::image_task::ImageTaskInfo> {
        crate::core::image_task::poll_image_task_info(provider, api_key, provider_task_id, base_url, task_id, prompt, created_at).await
    }
    /// Cancel an image generation task (best effort, local-only for now).
    pub async fn cancel_image_task_info(
        &self,
        provider: ImageModelProvider,
        api_key: String,
        provider_task_id: Option<String>,
        base_url: Option<String>,
        task_id: String,
        prompt: String,
        created_at: u64,
    ) -> HippoxResult<crate::core::image_task::ImageTaskInfo> {
        crate::core::image_task::cancel_image_task_info(provider, api_key, provider_task_id, base_url, task_id, prompt, created_at).await
    }
    /// Download the produced image(s) to `output_path`.
    ///
    /// # Arguments
    /// * `download_urls` - Remote URLs returned by the previous poll.
    /// * `output_path` - Directory where the image(s) will be saved.
    /// * `output_filename` - Optional filename; when multiple images are
    ///   downloaded and a name is given, an index suffix is appended.
    /// * `task_id` - Hippox-internal task id, carried over from persistence.
    /// * `provider` - Provider name, carried over from persistence.
    /// * `prompt` - Original prompt, carried over from persistence.
    /// * `created_at` - Original creation timestamp, carried over from persistence.
    ///
    /// # Returns
    /// `ImageTaskInfo` with `local_paths` populated on success.
    pub async fn download_image_task(
        &self,
        download_urls: Vec<String>,
        output_path: String,
        output_filename: Option<String>,
        task_id: String,
        provider: String,
        prompt: String,
        created_at: u64,
    ) -> HippoxResult<crate::core::image_task::ImageTaskInfo> {
        crate::core::image_task::download_image_task(download_urls, output_path, output_filename, task_id, provider, prompt, created_at).await
    }
    /// Submit an audio generation task and return immediately.
    ///
    /// # Arguments
    /// * `provider` - The audio model provider to use.
    /// * `api_key` - API key for the provider.
    /// * `prompt` - Text prompt for audio generation.
    /// * `options` - Optional generation options (voice, emotion, format, etc.).
    /// * `base_url` - Optional custom base URL for the provider.
    ///
    /// # Returns
    /// `AudioTaskInfo` containing the hippox task id, the provider task id,
    /// the current state, the download url (when available) and the usage.
    pub async fn submit_audio_task_info(
        &self,
        provider: AudioModelProvider,
        api_key: String,
        prompt: String,
        options: Option<AudioLLMOptions>,
        base_url: Option<String>,
    ) -> HippoxResult<crate::core::audio_task::AudioTaskInfo> {
        crate::core::audio_task::submit_audio_task_info(provider, api_key, prompt, options, base_url).await
    }
    /// Poll the provider once for the current state of an audio task.
    pub async fn poll_audio_task_info(
        &self,
        provider: AudioModelProvider,
        api_key: String,
        provider_task_id: String,
        base_url: Option<String>,
        task_id: String,
        prompt: String,
        created_at: u64,
    ) -> HippoxResult<crate::core::audio_task::AudioTaskInfo> {
        crate::core::audio_task::poll_audio_task_info(provider, api_key, provider_task_id, base_url, task_id, prompt, created_at).await
    }
    /// Cancel an audio generation task (best effort, local-only for now).
    pub async fn cancel_audio_task_info(
        &self,
        provider: AudioModelProvider,
        api_key: String,
        provider_task_id: Option<String>,
        base_url: Option<String>,
        task_id: String,
        prompt: String,
        created_at: u64,
    ) -> HippoxResult<crate::core::audio_task::AudioTaskInfo> {
        crate::core::audio_task::cancel_audio_task_info(provider, api_key, provider_task_id, base_url, task_id, prompt, created_at).await
    }
    /// Download the produced audio to `output_path / output_filename`.
    ///
    /// # Arguments
    /// * `download_url` - Remote URL returned by the previous poll (optional
    ///   for providers that return base64 directly).
    /// * `audio_base64` - Base64 payload returned by the previous poll
    ///   (optional for providers that return a URL).
    /// * `format` - Audio format, e.g. "mp3" / "wav".
    /// * `output_path` - Directory where the audio will be saved.
    /// * `output_filename` - Optional filename; when `None` a default
    ///   `{task_id}.{format}` name is used.
    /// * `task_id` - Hippox-internal task id, carried over from persistence.
    /// * `provider` - Provider name, carried over from persistence.
    /// * `prompt` - Original prompt, carried over from persistence.
    /// * `created_at` - Original creation timestamp, carried over from persistence.
    ///
    /// # Returns
    /// `AudioTaskInfo` with `local_paths` populated on success.
    pub async fn download_audio_task(
        &self,
        download_url: Option<String>,
        audio_base64: Option<String>,
        format: Option<String>,
        output_path: String,
        output_filename: Option<String>,
        task_id: String,
        provider: String,
        prompt: String,
        created_at: u64,
    ) -> HippoxResult<crate::core::audio_task::AudioTaskInfo> {
        crate::core::audio_task::download_audio_task(
            download_url,
            audio_base64,
            format,
            output_path,
            output_filename,
            task_id,
            provider,
            prompt,
            created_at,
        )
        .await
    }
    /// Heartbeat for the chat (LLM) channel.
    pub async fn heartbeat(&self) -> HippoxStringResult {
        let mut messages: Vec<ChatMessage> = Vec::new();
        messages.push(ChatMessage::user("hi"));
        match self.scheduler.chat_raw(messages).await {
            Ok(result) => {
                let usage = result.extract_usage();
                let input_tokens = usage.as_ref().map(|u| u.prompt_tokens as u64).unwrap_or(0);
                let output_tokens = usage.as_ref().map(|u| u.completion_tokens as u64).unwrap_or(0);
                INPUT_TOKEN_COUNT.fetch_add(input_tokens, std::sync::atomic::Ordering::Relaxed);
                OUTPUT_TOKEN_COUNT.fetch_add(output_tokens, std::sync::atomic::Ordering::Relaxed);
                HippoxResult::ok_with_tokens(result.text, input_tokens, output_tokens)
            }
            Err(e) => HippoxResult::network_error(e.to_string()),
        }
    }
    /// Heartbeat for the video channel.
    ///
    /// # Arguments
    /// * `provider` - The video model provider to probe.
    /// * `api_key` - API key for the provider. Must be non-empty.
    /// * `base_url` - Optional custom base URL override.
    pub async fn heartbeat_video(&self, provider: VideoModelProvider, api_key: Option<String>, base_url: Option<String>) -> HippoxStringResult {
        if VideoModelProvider::all().is_empty() {
            return HippoxResult::system_error("no video provider available".to_string());
        }
        let key = match api_key {
            Some(k) if !k.is_empty() => k,
            Some(_) => return HippoxResult::system_error(format!("video channel key empty: {:?}", provider)),
            None => return HippoxResult::system_error(format!("video channel key missing: {:?}", provider)),
        };
        let base_opt = base_url.as_deref().filter(|b| !b.trim().is_empty());
        match provider.probe(&key, base_opt).await {
            Ok(_) => HippoxResult::ok(format!("video channel ok: {:?}", provider)),
            Err(e) => HippoxResult::network_error(format!("video channel unreachable: {:?}: {}", provider, e)),
        }
    }
    /// Heartbeat for the image channel.
    ///
    /// # Arguments
    /// * `provider` - The image model provider to probe.
    /// * `api_key` - API key for the provider. Must be non-empty.
    /// * `base_url` - Optional custom base URL override.
    pub async fn heartbeat_image(&self, provider: ImageModelProvider, api_key: Option<String>, base_url: Option<String>) -> HippoxStringResult {
        if ImageModelProvider::all().is_empty() {
            return HippoxResult::system_error("no image provider available".to_string());
        }
        let key = match api_key {
            Some(k) if !k.is_empty() => k,
            Some(_) => return HippoxResult::system_error(format!("image channel key empty: {:?}", provider)),
            None => return HippoxResult::system_error(format!("image channel key missing: {:?}", provider)),
        };
        let base_opt = base_url.as_deref().filter(|b| !b.trim().is_empty());
        match provider.probe(&key, base_opt).await {
            Ok(_) => HippoxResult::ok(format!("image channel ok: {:?}", provider)),
            Err(e) => HippoxResult::network_error(format!("image channel unreachable: {:?}: {}", provider, e)),
        }
    }
    /// Heartbeat for the audio channel.
    ///
    /// # Arguments
    /// * `provider` - The audio model provider to probe.
    /// * `api_key` - API key for the provider. Must be non-empty.
    /// * `base_url` - Optional custom base URL override.
    pub async fn heartbeat_audio(&self, provider: AudioModelProvider, api_key: Option<String>, base_url: Option<String>) -> HippoxStringResult {
        if AudioModelProvider::all().is_empty() {
            return HippoxResult::system_error("no audio provider available".to_string());
        }
        let key = match api_key {
            Some(k) if !k.is_empty() => k,
            Some(_) => return HippoxResult::system_error(format!("audio channel key empty: {:?}", provider)),
            None => return HippoxResult::system_error(format!("audio channel key missing: {:?}", provider)),
        };
        let base_opt = base_url.as_deref().filter(|b| !b.trim().is_empty());
        match provider.probe(&key, base_opt).await {
            Ok(_) => HippoxResult::ok(format!("audio channel ok: {:?}", provider)),
            Err(e) => HippoxResult::network_error(format!("audio channel unreachable: {:?}: {}", provider, e)),
        }
    }
    /// List all available atomic drivers
    pub fn list_atomic_drivers(&self) -> HippoxStringResult {
        let drivers = get_all_drivers();
        if drivers.is_empty() {
            return HippoxResult::ok(t!("driver.no_drivers_available").to_string());
        }
        let mut result = String::new();
        for driver in drivers {
            let category = driver.category();
            let emoji = category.icon();
            result.push_str(&format!("   {} - **{}**: {}\n", emoji, driver.name(), driver.description()));
        }
        HippoxResult::ok(result)
    }
    /// Get all loaded atomic driver names
    pub fn get_driver_names(&self) -> HippoxBatchResult {
        HippoxResult::ok(list_drivers_names())
    }
    /// Check if there are any atomic drivers available
    pub fn has_atomic_drivers(&self) -> HippoxBoolResult {
        HippoxResult::ok(!list_drivers_names().is_empty())
    }
    /// Get the executor
    pub fn executor(&self) -> &Executor {
        &self.executor
    }
    /// Get the scheduler
    pub fn scheduler(&self) -> &DriverScheduler {
        &self.scheduler
    }
    /// Update configuration
    pub fn update_config<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut HippoxConfig),
    {
        crate::config::update_config(f)
    }
    /// Get configuration
    pub fn get_config(&self) -> HippoxConfig {
        crate::config::get_config()
    }
    /// Get current global input token count
    ///
    /// # Returns
    /// The total input token count as u64
    ///
    /// # Example
    /// ```
    /// let hippox = Hippox::builder(ModelProvider::OpenAI).build().await?;
    /// let input_tokens = hippox.get_current_input_token_count();
    /// println!("Total input tokens: {}", input_tokens);
    /// ```
    pub fn get_current_input_token_count(&self) -> u64 {
        INPUT_TOKEN_COUNT.load(std::sync::atomic::Ordering::Relaxed)
    }
    /// Get current global output token count
    ///
    /// # Returns
    /// The total output token count as u64
    ///
    /// # Example
    /// ```
    /// let hippox = Hippox::builder(ModelProvider::OpenAI).build().await?;
    /// let output_tokens = hippox.get_current_output_token_count();
    /// println!("Total output tokens: {}", output_tokens);
    /// ```
    pub fn get_current_output_token_count(&self) -> u64 {
        OUTPUT_TOKEN_COUNT.load(std::sync::atomic::Ordering::Relaxed)
    }
    /// Storage task pool to a JSON file and remove completed tasks from memory
    ///
    /// # Arguments
    /// * `path` - The file path to save the JSON file (e.g., "./task_pool.json")
    ///
    /// # Returns
    /// `HippoxVoidResult` - Ok(()) on success, or error on failure
    ///
    /// # Example
    /// ```ignore
    /// let hippox = Hippox::builder(ModelProvider::OpenAI).build().await?;
    /// hippox.storage_task_pool("./tasks_backup.json".to_string());
    /// ```
    pub fn storage_task_pool(&self, path: String) -> HippoxVoidResult {
        use futures::executor::block_on;
        // Get all terminal state tasks and remove them from pool atomically
        let (exported_tasks, removed_count) = block_on(async {
            let mut pool = tasks::TASK_POOL.write().await;
            // Collect terminal state tasks
            let terminal_tasks: Vec<tasks::Task> = pool
                .tasks
                .values()
                .filter(|task| matches!(task.status, TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled | TaskStatus::Timeout))
                .cloned()
                .collect();
            let removed_count = terminal_tasks.len();
            // Remove them from the pool
            for task in &terminal_tasks {
                pool.tasks.remove(&task.id);
                // Also clean up from pending_queue and running_tasks just in case
                pool.pending_queue.retain(|id| id != &task.id);
                pool.running_tasks.retain(|id| id != &task.id);
            }
            (terminal_tasks, removed_count)
        });
        if exported_tasks.is_empty() {
            info!("No terminal state tasks to backup and remove");
            return HippoxResult::ok(());
        }
        let json_data = json!({
            "export_time": chrono::Local::now().to_rfc3339(),
            "total_count": exported_tasks.len(),
            "tasks": exported_tasks.iter().map(|task| {
                json!({
                    "id": task.id,
                    "task_type": task.task_type,
                    "input": task.input,
                    "status": format!("{:?}", task.status),
                    "final_output": task.final_output,
                    "error": task.error,
                    "created_at": task.created_at,
                    "started_at": task.started_at,
                    "completed_at": task.completed_at,
                    "duration_ms": task.duration_ms,
                    "input_token_count": task.input_token_count,
                    "output_token_count": task.output_token_count,
                    "steps": task.steps.iter().map(|step| {
                        json!({
                            "driver_name": step.driver_name,
                            "status": format!("{:?}", step.status),
                            "output": step.output,
                            "error": step.error,
                            "duration_ms": step.duration_ms,
                        })
                    }).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        });
        let json_string = match serde_json::to_string_pretty(&json_data) {
            Ok(s) => s,
            Err(e) => {
                // If serialization fails, the tasks have already been removed
                // Log error but return error result
                tracing::error!("Failed to serialize tasks: {}", e);
                return HippoxResult::system_error(format!("Failed to serialize tasks: {}", e));
            }
        };
        match fs::write(&path, json_string) {
            Ok(_) => {
                info!("Successfully backed up and removed {} terminal tasks to: {}", removed_count, path);
                HippoxResult::ok(())
            }
            Err(e) => {
                // File write failed, but tasks are already removed!
                // This is a problem - data loss has occurred.
                tracing::error!("Failed to write backup file after removing tasks! Data loss occurred. Path: {}, Error: {}", path, e);
                HippoxResult::system_error(format!("Failed to write file {} after removing tasks (data may be lost): {}", path, e))
            }
        }
    }
}
