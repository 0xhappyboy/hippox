use std::collections::HashMap;

use langhub::types::ModelProvider;

use crate::{Hippox, HippoxConfig, IdentityInformation, WorkflowMode};

/// Builder for creating Hippox instances
pub struct HippoxBuilder {
    provider: ModelProvider,
    api_key: Option<String>,
    extra_keys: Option<HashMap<String, String>>,
    config: HippoxConfig,
    workflow_mode: WorkflowMode,
}

impl HippoxBuilder {
    /// Create a new builder with required provider
    pub fn new(provider: ModelProvider) -> Self {
        Self {
            provider,
            api_key: None,
            extra_keys: None,
            config: HippoxConfig::default(),
            workflow_mode: WorkflowMode::default(),
        }
    }

    /// Set API key
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set extra keys (e.g., for Azure, custom endpoints)
    pub fn extra_keys(mut self, keys: HashMap<String, String>) -> Self {
        self.extra_keys = Some(keys);
        self
    }

    /// Set language
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.config.lang = lang.into();
        self
    }

    /// Set workflow mode
    pub fn workflow_mode(mut self, mode: WorkflowMode) -> Self {
        self.workflow_mode = mode;
        self
    }

    /// Set identity with a closure
    pub fn identity(mut self, f: impl FnOnce(&mut IdentityInformation)) -> Self {
        f(&mut self.config.identity_information);
        self
    }

    /// Build the Hippox instance
    pub async fn build(self) -> anyhow::Result<Hippox> {
        Hippox::with_workflow_mode(
            self.provider,
            self.api_key,
            self.extra_keys,
            Some(self.config),
            self.workflow_mode,
        )
        .await
    }
}

impl Hippox {
    /// Create a new builder
    pub fn builder(provider: ModelProvider) -> HippoxBuilder {
        HippoxBuilder::new(provider)
    }
}
