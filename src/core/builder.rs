use std::collections::HashMap;

use langhub::types::ModelProvider;

use crate::{
    DockerConfig, Hippox, HippoxConfig, IdentityInformation, K8sConfig, MySQLConfig,
    PostgreSQLConfig, RedisConfig, WorkflowMode,
};

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

    /// Add a PostgreSQL instance
    pub fn add_postgresql(mut self, config: PostgreSQLConfig) -> Self {
        self.config.add_postgresql_instance(config);
        self
    }

    /// Add a MySQL instance
    pub fn add_mysql(mut self, config: MySQLConfig) -> Self {
        self.config.add_mysql_instance(config);
        self
    }

    /// Add a Redis instance
    pub fn add_redis(mut self, config: RedisConfig) -> Self {
        self.config.add_redis_instance(config);
        self
    }

    /// Add a Docker instance
    pub fn add_docker(mut self, config: DockerConfig) -> Self {
        self.config.add_docker_instance(config);
        self
    }

    /// Add a K8s instance
    pub fn add_k8s(mut self, config: K8sConfig) -> Self {
        self.config.add_k8s_instance(config);
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
