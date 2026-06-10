//! Core configuration structure and global state

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

use super::instances::*;

/// Global static configuration instance
pub(crate) static HIPPOX_CORE_CONFIG: Lazy<RwLock<HippoxConfig>> =
    Lazy::new(|| RwLock::new(HippoxConfig::default()));

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct IdentityInformation {
    // Name, Default: Hippox
    pub name: Option<String>,
    // Sex
    pub sex: Option<String>,
    // age
    pub age: Option<String>,
    // Species
    pub species: Option<String>,
    // Role/Profession (e.g., "assistant", "teacher", "life coach")
    pub role: Option<String>,
    // Personality traits (e.g., "friendly", "humorous", "professional")
    pub personality: Option<String>,
    // Tone style (e.g., "casual", "formal", "poetic")
    pub tone_style: Option<String>,
    // Knowledge scope (e.g., "general", "medical", "programming")
    pub knowledge_scope: Option<String>,
    // Catchphrase / habitual expression (e.g., "Haha", "I see")
    pub catchphrase: Option<String>,
    // Taboos / prohibited topics (e.g., "no politics", "no medical advice")
    pub taboos: Option<String>,
}

impl Default for IdentityInformation {
    fn default() -> Self {
        Self {
            name: Some("Hippox".to_string()),
            sex: None,
            age: None,
            species: None,
            role: None,
            personality: None,
            tone_style: None,
            knowledge_scope: None,
            catchphrase: None,
            taboos: None,
        }
    }
}

/// Hippox global configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct HippoxConfig {
    // Application settings
    pub lang: String,
    // Identity information
    pub identity_information: IdentityInformation,
    // Database configurations (multiple instances)
    pub postgresql_instances: HashMap<String, PostgreSQLConfig>,
    pub mysql_instances: HashMap<String, MySQLConfig>,
    pub redis_instances: HashMap<String, RedisConfig>,
    pub sqlite_instances: HashMap<String, SQLiteConfig>,

    // Container configurations (multiple instances)
    pub docker_instances: HashMap<String, DockerConfig>,

    // Kubernetes configurations (multiple clusters)
    pub k8s_instances: HashMap<String, K8sConfig>,

    // Network configurations (multiple instances)
    pub tcp_instances: HashMap<String, TCPConfig>,
    pub udp_instances: HashMap<String, UDPConfig>,
    pub ftp_instances: HashMap<String, FTPConfig>,

    // Notification configurations (multiple instances)
    pub smtp_instances: HashMap<String, SMTPConfig>,
    pub telegram_instances: HashMap<String, TelegramConfig>,
    pub dingtalk_instances: HashMap<String, DingTalkConfig>,
    pub feishu_instances: HashMap<String, FeishuConfig>,
    pub wecom_instances: HashMap<String, WeComConfig>,

    // GitHub configurations (multiple instances)
    pub github_instances: HashMap<String, GitHubConfig>,
}

impl Default for HippoxConfig {
    fn default() -> Self {
        Self {
            lang: "en".to_string(),
            identity_information: IdentityInformation::default(),
            postgresql_instances: HashMap::new(),
            mysql_instances: HashMap::new(),
            redis_instances: HashMap::new(),
            sqlite_instances: HashMap::new(),
            docker_instances: HashMap::new(),
            k8s_instances: HashMap::new(),
            tcp_instances: HashMap::new(),
            udp_instances: HashMap::new(),
            ftp_instances: HashMap::new(),
            smtp_instances: HashMap::new(),
            telegram_instances: HashMap::new(),
            dingtalk_instances: HashMap::new(),
            feishu_instances: HashMap::new(),
            wecom_instances: HashMap::new(),
            github_instances: HashMap::new(),
        }
    }
}

impl HippoxConfig {
    /// Get identity information reference
    pub fn get_identity(&self) -> &IdentityInformation {
        &self.identity_information
    }

    /// Get mutable identity information reference
    pub fn get_identity_mut(&mut self) -> &mut IdentityInformation {
        &mut self.identity_information
    }

    /// Update identity information
    pub fn update_identity<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut IdentityInformation),
    {
        f(&mut self.identity_information);
        self
    }

    /// Load from TOML configuration file
    pub fn load_from_toml_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: HippoxConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load from JSON configuration file
    pub fn load_from_json_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: HippoxConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    // ========== PostgreSQL ==========
    pub fn add_postgresql_instance(&mut self, config: PostgreSQLConfig) -> String {
        let id = config.id.clone();
        self.postgresql_instances.insert(id.clone(), config);
        id
    }

    pub fn get_postgresql_instance(&self, id: &str) -> Option<&PostgreSQLConfig> {
        self.postgresql_instances.get(id)
    }

    pub fn remove_postgresql_instance(&mut self, id: &str) -> Option<PostgreSQLConfig> {
        self.postgresql_instances.remove(id)
    }

    pub fn list_postgresql_instances(&self) -> Vec<&PostgreSQLConfig> {
        self.postgresql_instances.values().collect()
    }

    pub fn has_postgresql(&self) -> bool {
        !self.postgresql_instances.is_empty()
    }

    // ========== MySQL ==========
    pub fn add_mysql_instance(&mut self, config: MySQLConfig) -> String {
        let id = config.id.clone();
        self.mysql_instances.insert(id.clone(), config);
        id
    }

    pub fn get_mysql_instance(&self, id: &str) -> Option<&MySQLConfig> {
        self.mysql_instances.get(id)
    }

    pub fn remove_mysql_instance(&mut self, id: &str) -> Option<MySQLConfig> {
        self.mysql_instances.remove(id)
    }

    pub fn list_mysql_instances(&self) -> Vec<&MySQLConfig> {
        self.mysql_instances.values().collect()
    }

    pub fn has_mysql(&self) -> bool {
        !self.mysql_instances.is_empty()
    }

    // ========== Redis ==========
    pub fn add_redis_instance(&mut self, config: RedisConfig) -> String {
        let id = config.id.clone();
        self.redis_instances.insert(id.clone(), config);
        id
    }

    pub fn get_redis_instance(&self, id: &str) -> Option<&RedisConfig> {
        self.redis_instances.get(id)
    }

    pub fn remove_redis_instance(&mut self, id: &str) -> Option<RedisConfig> {
        self.redis_instances.remove(id)
    }

    pub fn list_redis_instances(&self) -> Vec<&RedisConfig> {
        self.redis_instances.values().collect()
    }

    pub fn has_redis(&self) -> bool {
        !self.redis_instances.is_empty()
    }

    // ========== SQLite ==========
    pub fn add_sqlite_instance(&mut self, config: SQLiteConfig) -> String {
        let id = config.id.clone();
        self.sqlite_instances.insert(id.clone(), config);
        id
    }

    pub fn get_sqlite_instance(&self, id: &str) -> Option<&SQLiteConfig> {
        self.sqlite_instances.get(id)
    }

    pub fn remove_sqlite_instance(&mut self, id: &str) -> Option<SQLiteConfig> {
        self.sqlite_instances.remove(id)
    }

    pub fn list_sqlite_instances(&self) -> Vec<&SQLiteConfig> {
        self.sqlite_instances.values().collect()
    }

    pub fn has_sqlite(&self) -> bool {
        !self.sqlite_instances.is_empty()
    }

    // ========== Docker ==========
    pub fn add_docker_instance(&mut self, config: DockerConfig) -> String {
        let id = config.id.clone();
        self.docker_instances.insert(id.clone(), config);
        id
    }

    pub fn get_docker_instance(&self, id: &str) -> Option<&DockerConfig> {
        self.docker_instances.get(id)
    }

    pub fn remove_docker_instance(&mut self, id: &str) -> Option<DockerConfig> {
        self.docker_instances.remove(id)
    }

    pub fn list_docker_instances(&self) -> Vec<&DockerConfig> {
        self.docker_instances.values().collect()
    }

    pub fn has_docker(&self) -> bool {
        !self.docker_instances.is_empty()
    }

    // ========== Kubernetes ==========
    pub fn add_k8s_instance(&mut self, config: K8sConfig) -> String {
        let id = config.id.clone();
        self.k8s_instances.insert(id.clone(), config);
        id
    }

    pub fn get_k8s_instance(&self, id: &str) -> Option<&K8sConfig> {
        self.k8s_instances.get(id)
    }

    pub fn remove_k8s_instance(&mut self, id: &str) -> Option<K8sConfig> {
        self.k8s_instances.remove(id)
    }

    pub fn list_k8s_instances(&self) -> Vec<&K8sConfig> {
        self.k8s_instances.values().collect()
    }

    pub fn has_k8s(&self) -> bool {
        !self.k8s_instances.is_empty()
    }

    // ========== TCP ==========
    pub fn add_tcp_instance(&mut self, config: TCPConfig) -> String {
        let id = config.id.clone();
        self.tcp_instances.insert(id.clone(), config);
        id
    }

    pub fn get_tcp_instance(&self, id: &str) -> Option<&TCPConfig> {
        self.tcp_instances.get(id)
    }

    pub fn remove_tcp_instance(&mut self, id: &str) -> Option<TCPConfig> {
        self.tcp_instances.remove(id)
    }

    pub fn list_tcp_instances(&self) -> Vec<&TCPConfig> {
        self.tcp_instances.values().collect()
    }

    pub fn has_tcp(&self) -> bool {
        !self.tcp_instances.is_empty()
    }

    // ========== UDP ==========
    pub fn add_udp_instance(&mut self, config: UDPConfig) -> String {
        let id = config.id.clone();
        self.udp_instances.insert(id.clone(), config);
        id
    }

    pub fn get_udp_instance(&self, id: &str) -> Option<&UDPConfig> {
        self.udp_instances.get(id)
    }

    pub fn remove_udp_instance(&mut self, id: &str) -> Option<UDPConfig> {
        self.udp_instances.remove(id)
    }

    pub fn list_udp_instances(&self) -> Vec<&UDPConfig> {
        self.udp_instances.values().collect()
    }

    pub fn has_udp(&self) -> bool {
        !self.udp_instances.is_empty()
    }

    // ========== FTP ==========
    pub fn add_ftp_instance(&mut self, config: FTPConfig) -> String {
        let id = config.id.clone();
        self.ftp_instances.insert(id.clone(), config);
        id
    }

    pub fn get_ftp_instance(&self, id: &str) -> Option<&FTPConfig> {
        self.ftp_instances.get(id)
    }

    pub fn remove_ftp_instance(&mut self, id: &str) -> Option<FTPConfig> {
        self.ftp_instances.remove(id)
    }

    pub fn list_ftp_instances(&self) -> Vec<&FTPConfig> {
        self.ftp_instances.values().collect()
    }

    pub fn has_ftp(&self) -> bool {
        !self.ftp_instances.is_empty()
    }

    // ========== SMTP ==========
    pub fn add_smtp_instance(&mut self, config: SMTPConfig) -> String {
        let id = config.id.clone();
        self.smtp_instances.insert(id.clone(), config);
        id
    }

    pub fn get_smtp_instance(&self, id: &str) -> Option<&SMTPConfig> {
        self.smtp_instances.get(id)
    }

    pub fn remove_smtp_instance(&mut self, id: &str) -> Option<SMTPConfig> {
        self.smtp_instances.remove(id)
    }

    pub fn list_smtp_instances(&self) -> Vec<&SMTPConfig> {
        self.smtp_instances.values().collect()
    }

    pub fn has_smtp(&self) -> bool {
        !self.smtp_instances.is_empty()
    }

    // ========== Telegram ==========
    pub fn add_telegram_instance(&mut self, config: TelegramConfig) -> String {
        let id = config.id.clone();
        self.telegram_instances.insert(id.clone(), config);
        id
    }

    pub fn get_telegram_instance(&self, id: &str) -> Option<&TelegramConfig> {
        self.telegram_instances.get(id)
    }

    pub fn remove_telegram_instance(&mut self, id: &str) -> Option<TelegramConfig> {
        self.telegram_instances.remove(id)
    }

    pub fn list_telegram_instances(&self) -> Vec<&TelegramConfig> {
        self.telegram_instances.values().collect()
    }

    pub fn has_telegram(&self) -> bool {
        !self.telegram_instances.is_empty()
    }

    // ========== DingTalk ==========
    pub fn add_dingtalk_instance(&mut self, config: DingTalkConfig) -> String {
        let id = config.id.clone();
        self.dingtalk_instances.insert(id.clone(), config);
        id
    }

    pub fn get_dingtalk_instance(&self, id: &str) -> Option<&DingTalkConfig> {
        self.dingtalk_instances.get(id)
    }

    pub fn remove_dingtalk_instance(&mut self, id: &str) -> Option<DingTalkConfig> {
        self.dingtalk_instances.remove(id)
    }

    pub fn list_dingtalk_instances(&self) -> Vec<&DingTalkConfig> {
        self.dingtalk_instances.values().collect()
    }

    pub fn has_dingtalk(&self) -> bool {
        !self.dingtalk_instances.is_empty()
    }

    // ========== Feishu ==========
    pub fn add_feishu_instance(&mut self, config: FeishuConfig) -> String {
        let id = config.id.clone();
        self.feishu_instances.insert(id.clone(), config);
        id
    }

    pub fn get_feishu_instance(&self, id: &str) -> Option<&FeishuConfig> {
        self.feishu_instances.get(id)
    }

    pub fn remove_feishu_instance(&mut self, id: &str) -> Option<FeishuConfig> {
        self.feishu_instances.remove(id)
    }

    pub fn list_feishu_instances(&self) -> Vec<&FeishuConfig> {
        self.feishu_instances.values().collect()
    }

    pub fn has_feishu(&self) -> bool {
        !self.feishu_instances.is_empty()
    }

    // ========== WeCom ==========
    pub fn add_wecom_instance(&mut self, config: WeComConfig) -> String {
        let id = config.id.clone();
        self.wecom_instances.insert(id.clone(), config);
        id
    }

    pub fn get_wecom_instance(&self, id: &str) -> Option<&WeComConfig> {
        self.wecom_instances.get(id)
    }

    pub fn remove_wecom_instance(&mut self, id: &str) -> Option<WeComConfig> {
        self.wecom_instances.remove(id)
    }

    pub fn list_wecom_instances(&self) -> Vec<&WeComConfig> {
        self.wecom_instances.values().collect()
    }

    pub fn has_wecom(&self) -> bool {
        !self.wecom_instances.is_empty()
    }

    // ========== GitHub ==========
    pub fn add_github_instance(&mut self, config: GitHubConfig) -> String {
        let id = config.id.clone();
        self.github_instances.insert(id.clone(), config);
        id
    }

    pub fn get_github_instance(&self, id: &str) -> Option<&GitHubConfig> {
        self.github_instances.get(id)
    }

    pub fn remove_github_instance(&mut self, id: &str) -> Option<GitHubConfig> {
        self.github_instances.remove(id)
    }

    pub fn list_github_instances(&self) -> Vec<&GitHubConfig> {
        self.github_instances.values().collect()
    }

    pub fn has_github(&self) -> bool {
        !self.github_instances.is_empty()
    }
}

/// Get a clone of the global configuration
pub fn get_config() -> HippoxConfig {
    HIPPOX_CORE_CONFIG.read().unwrap().clone()
}

/// Update config with a closure
pub fn update_config<F>(f: F) -> anyhow::Result<()>
where
    F: FnOnce(&mut HippoxConfig),
{
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    f(&mut global);
    Ok(())
}

/// Get current language setting
pub fn get_lang() -> String {
    HIPPOX_CORE_CONFIG.read().unwrap().lang.clone()
}

/// Set language
pub fn set_lang(lang: String) -> anyhow::Result<()> {
    let mut config = HIPPOX_CORE_CONFIG.write().unwrap();
    config.lang = lang;
    Ok(())
}

/// Get the global Hippox core configuration (alias for get_config for backward compatibility)
pub fn get_hippox_core_config() -> HippoxConfig {
    HIPPOX_CORE_CONFIG.read().unwrap().clone()
}
