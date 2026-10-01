use crate::{Hippox, HippoxConfig, IdentityInformation};
use langhub::{
    AudioLLMClient, AudioLLMConfig, ImageLLMClient, ImageLLMConfig, VideoLLMClient, VideoLLMConfig, audio::AudioModelProvider,
    image::ImageModelProvider, types::ModelProvider, video::VideoModelProvider,
};
use std::collections::HashMap;
pub struct HippoxModel {
    /// The LLM (chat) core instance, present only when an LLM provider was set.
    pub llm: Option<Hippox>,
    /// The image client, present only when an image provider + config were set.
    pub image: Option<ImageLLMClient>,
    /// The video client, present only when a video provider + config were set.
    pub video: Option<VideoLLMClient>,
    /// The audio client, present only when an audio provider + config were set.
    pub audio: Option<AudioLLMClient>,
}
/// Builder for creating Hippox instances.
pub struct HippoxBuilder {
    // LLM (chat) modality
    llm_provider: Option<ModelProvider>,
    llm_api_key: Option<String>,
    llm_extra_keys: Option<HashMap<String, String>>,
    // Image modality
    image_provider: Option<ImageModelProvider>,
    image_config: Option<ImageLLMConfig>,
    // Video modality
    video_provider: Option<VideoModelProvider>,
    video_config: Option<VideoLLMConfig>,
    // Audio modality
    audio_provider: Option<AudioModelProvider>,
    audio_config: Option<AudioLLMConfig>,
    // Shared runtime config
    config: HippoxConfig,
}
impl HippoxBuilder {
    /// Create a new builder for the LLM (chat) modality.
    pub fn new(provider: ModelProvider) -> Self {
        Self {
            llm_provider: Some(provider),
            llm_api_key: None,
            llm_extra_keys: None,
            image_provider: None,
            image_config: None,
            video_provider: None,
            video_config: None,
            audio_provider: None,
            audio_config: None,
            config: HippoxConfig::default(),
        }
    }
    /// Create a new builder for the image modality.
    pub fn new_image(provider: ImageModelProvider, config: ImageLLMConfig) -> Self {
        Self {
            llm_provider: None,
            llm_api_key: None,
            llm_extra_keys: None,
            image_provider: Some(provider),
            image_config: Some(config),
            video_provider: None,
            video_config: None,
            audio_provider: None,
            audio_config: None,
            config: HippoxConfig::default(),
        }
    }
    /// Create a new builder for the video modality.
    pub fn new_video(provider: VideoModelProvider, config: VideoLLMConfig) -> Self {
        Self {
            llm_provider: None,
            llm_api_key: None,
            llm_extra_keys: None,
            image_provider: None,
            image_config: None,
            video_provider: Some(provider),
            video_config: Some(config),
            audio_provider: None,
            audio_config: None,
            config: HippoxConfig::default(),
        }
    }
    /// Create a new builder for the audio modality.
    pub fn new_audio(provider: AudioModelProvider, config: AudioLLMConfig) -> Self {
        Self {
            llm_provider: None,
            llm_api_key: None,
            llm_extra_keys: None,
            image_provider: None,
            image_config: None,
            video_provider: None,
            video_config: None,
            audio_provider: Some(provider),
            audio_config: Some(config),
            config: HippoxConfig::default(),
        }
    }
    /// Set the LLM API key.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.llm_api_key = Some(key.into());
        self
    }
    /// Set LLM extra keys (e.g., for Azure, custom endpoints).
    pub fn extra_keys(mut self, keys: HashMap<String, String>) -> Self {
        self.llm_extra_keys = Some(keys);
        self
    }
    /// Set language.
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.config.lang = lang.into();
        self
    }
    /// Set identity with a closure.
    pub fn identity(mut self, f: impl FnOnce(&mut IdentityInformation)) -> Self {
        f(&mut self.config.identity_information);
        self
    }
    /// Set (or override) the image modality.
    pub fn image(mut self, provider: ImageModelProvider, config: ImageLLMConfig) -> Self {
        self.image_provider = Some(provider);
        self.image_config = Some(config);
        self
    }
    /// Set (or override) the video modality.
    pub fn video(mut self, provider: VideoModelProvider, config: VideoLLMConfig) -> Self {
        self.video_provider = Some(provider);
        self.video_config = Some(config);
        self
    }
    /// Set (or override) the audio modality.
    pub fn audio(mut self, provider: AudioModelProvider, config: AudioLLMConfig) -> Self {
        self.audio_provider = Some(provider);
        self.audio_config = Some(config);
        self
    }
    /// Build the `Hippox` core instance (LLM side only).
    pub async fn build(self) -> anyhow::Result<Hippox> {
        let provider = self
            .llm_provider
            .ok_or_else(|| anyhow::anyhow!("LLM provider is required for `build()`; use `build_with_model()` for modality-only builders"))?;
        Hippox::with_workflow_mode(provider, self.llm_api_key, self.llm_extra_keys, Some(self.config)).await
    }
    /// Build every configured modality and return them as a [`HippoxModel`].
    ///
    /// # Example
    /// ```ignore
    /// let model = Hippox::builder(ModelProvider::OpenAI)
    ///     .api_key("sk-xxx")
    ///     .image(
    ///         ImageModelProvider::Seedream,
    ///         ImageLLMConfig::new().seedream("ark-key".to_string()),
    ///     )
    ///     .build_with_model().await?;
    ///
    /// if let Some(llm) = &model.llm {
    ///     // use the LLM core
    /// }
    /// if let Some(image) = &model.image {
    ///     let result = image.generate("a cat").await?;
    /// }
    /// ```
    pub async fn build_with_model(self) -> anyhow::Result<HippoxModel> {
        // Build the LLM core only when an LLM provider exists
        let llm = match self.llm_provider {
            Some(provider) => Some(Hippox::with_workflow_mode(provider, self.llm_api_key, self.llm_extra_keys, Some(self.config)).await?),
            None => None,
        };
        // Build image client (own provider enum + own config type)
        let image = match (self.image_provider, self.image_config) {
            (Some(provider), Some(config)) => Some(Hippox::new_llm_image_with_config(provider, &config)?),
            _ => None,
        };
        // Build video client (own provider enum + own config type)
        let video = match (self.video_provider, self.video_config) {
            (Some(provider), Some(config)) => Some(Hippox::new_llm_video_with_config(provider, &config)?),
            _ => None,
        };
        // Build audio client (own provider enum + own config type)
        let audio = match (self.audio_provider, self.audio_config) {
            (Some(provider), Some(config)) => Some(Hippox::new_llm_audio_with_config(provider, &config)?),
            _ => None,
        };
        Ok(HippoxModel { llm, image, video, audio })
    }
}
impl Hippox {
    /// Create a new builder for the LLM (chat) modality.
    pub fn builder(provider: ModelProvider) -> HippoxBuilder {
        HippoxBuilder::new(provider)
    }
    /// Create a new builder for the image modality.
    ///
    /// # Example
    /// ```ignore
    /// let model = Hippox::builder_image(
    ///     ImageModelProvider::Seedream,
    ///     ImageLLMConfig::new().seedream("ark-key".to_string()),
    /// ).build_with_model().await?;
    /// let image = model.image.expect("image client");
    /// ```
    pub fn builder_image(provider: ImageModelProvider, config: ImageLLMConfig) -> HippoxBuilder {
        HippoxBuilder::new_image(provider, config)
    }
    /// Create a new builder for the video modality.
    ///
    /// # Example
    /// ```ignore
    /// let model = Hippox::builder_video(
    ///     VideoModelProvider::Seedance,
    ///     VideoLLMConfig::new().seedance("ark-key".to_string()),
    /// ).build_with_model().await?;
    /// let video = model.video.expect("video client");
    /// ```
    pub fn builder_video(provider: VideoModelProvider, config: VideoLLMConfig) -> HippoxBuilder {
        HippoxBuilder::new_video(provider, config)
    }
    /// Create a new builder for the audio modality.
    ///
    /// # Example
    /// ```ignore
    /// let model = Hippox::builder_audio(
    ///     AudioModelProvider::QwenTts,
    ///     AudioLLMConfig::new().qwen_tts("dashscope-key".to_string()),
    /// ).build_with_model().await?;
    /// let audio = model.audio.expect("audio client");
    /// ```
    pub fn builder_audio(provider: AudioModelProvider, config: AudioLLMConfig) -> HippoxBuilder {
        HippoxBuilder::new_audio(provider, config)
    }
}
