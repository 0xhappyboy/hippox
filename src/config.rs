use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

/// Global static configuration instance
pub(crate) static HIPPOX_CORE_CONFIG: Lazy<RwLock<HippoxConfig>> =
    Lazy::new(|| RwLock::new(HippoxConfig::default()));

/// PostgreSQL configuration for a single instance
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PostgreSQLConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub pool_size: usize,
    pub timeout: u64,
}

impl PostgreSQLConfig {
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        host: String,
        port: u16,
        database: String,
        username: String,
        password: String,
    ) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            host,
            port,
            database,
            username,
            password,
            pool_size: 10,
            timeout: 30,
        }
    }

    pub fn with_pool_size(mut self, pool_size: usize) -> Self {
        self.pool_size = pool_size;
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for PostgreSQLConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            host: String::new(),
            port: 5432,
            database: String::new(),
            username: String::new(),
            password: String::new(),
            pool_size: 10,
            timeout: 30,
        }
    }
}

/// MySQL configuration for a single instance
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MySQLConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub pool_size: usize,
    pub timeout: u64,
}

impl MySQLConfig {
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        host: String,
        port: u16,
        database: String,
        username: String,
        password: String,
    ) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            host,
            port,
            database,
            username,
            password,
            pool_size: 10,
            timeout: 30,
        }
    }

    pub fn with_pool_size(mut self, pool_size: usize) -> Self {
        self.pool_size = pool_size;
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for MySQLConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            host: String::new(),
            port: 3306,
            database: String::new(),
            username: String::new(),
            password: String::new(),
            pool_size: 10,
            timeout: 30,
        }
    }
}

/// Redis configuration for a single instance
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RedisConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub host: String,
    pub port: u16,
    pub password: String,
    pub db: usize,
    pub pool_size: usize,
    pub timeout: u64,
}

impl RedisConfig {
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        host: String,
        port: u16,
    ) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            host,
            port,
            password: String::new(),
            db: 0,
            pool_size: 10,
            timeout: 30,
        }
    }

    pub fn with_password(mut self, password: String) -> Self {
        self.password = password;
        self
    }

    pub fn with_db(mut self, db: usize) -> Self {
        self.db = db;
        self
    }

    pub fn with_pool_size(mut self, pool_size: usize) -> Self {
        self.pool_size = pool_size;
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            host: String::new(),
            port: 6379,
            password: String::new(),
            db: 0,
            pool_size: 10,
            timeout: 30,
        }
    }
}

/// SQLite configuration for a single instance
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SQLiteConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub pool_size: usize,
    pub timeout: u64,
}

impl SQLiteConfig {
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        path: String,
    ) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            path,
            pool_size: 5,
            timeout: 30,
        }
    }

    pub fn with_pool_size(mut self, pool_size: usize) -> Self {
        self.pool_size = pool_size;
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for SQLiteConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            path: String::new(),
            pool_size: 5,
            timeout: 30,
        }
    }
}

/// Docker configuration for a single instance
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DockerConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub host: String,
    pub api_version: String,
    pub timeout: u64,
    pub tls_verify: bool,
    pub cert_path: String,
}

impl DockerConfig {
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        host: String,
    ) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            host,
            api_version: String::new(),
            timeout: 30,
            tls_verify: false,
            cert_path: String::new(),
        }
    }

    pub fn with_api_version(mut self, api_version: String) -> Self {
        self.api_version = api_version;
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_tls(mut self, verify: bool, cert_path: String) -> Self {
        self.tls_verify = verify;
        self.cert_path = cert_path;
        self
    }
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            host: "unix:///var/run/docker.sock".to_string(),
            api_version: String::new(),
            timeout: 30,
            tls_verify: false,
            cert_path: String::new(),
        }
    }
}

/// Kubernetes configuration for a single cluster
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct K8sConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kubeconfig: String,
    pub context: String,
    pub namespace: String,
    pub api_server: String,
    pub api_token: String,
    pub timeout: u64,
    pub insecure: bool,
    pub ca_cert: String,
    pub client_cert: String,
    pub client_key: String,
}

impl K8sConfig {
    pub fn new(id: String, name: Option<String>, description: Option<String>) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            kubeconfig: String::new(),
            context: String::new(),
            namespace: "default".to_string(),
            api_server: String::new(),
            api_token: String::new(),
            timeout: 30,
            insecure: false,
            ca_cert: String::new(),
            client_cert: String::new(),
            client_key: String::new(),
        }
    }

    pub fn with_kubeconfig(mut self, kubeconfig: String) -> Self {
        self.kubeconfig = kubeconfig;
        self
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = context;
        self
    }

    pub fn with_namespace(mut self, namespace: String) -> Self {
        self.namespace = namespace;
        self
    }

    pub fn with_api_server(mut self, api_server: String, token: String) -> Self {
        self.api_server = api_server;
        self.api_token = token;
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_insecure(mut self, insecure: bool) -> Self {
        self.insecure = insecure;
        self
    }

    pub fn with_ca_cert(mut self, ca_cert: String) -> Self {
        self.ca_cert = ca_cert;
        self
    }

    pub fn with_client_cert(mut self, client_cert: String, client_key: String) -> Self {
        self.client_cert = client_cert;
        self.client_key = client_key;
        self
    }
}

impl Default for K8sConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            kubeconfig: String::new(),
            context: String::new(),
            namespace: "default".to_string(),
            api_server: String::new(),
            api_token: String::new(),
            timeout: 30,
            insecure: false,
            ca_cert: String::new(),
            client_cert: String::new(),
            client_key: String::new(),
        }
    }
}

/// TCP configuration for a single instance
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TCPConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub host: String,
    pub port: u16,
    pub timeout: u64,
    pub encoding: String,
}

impl TCPConfig {
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        host: String,
        port: u16,
    ) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            host,
            port,
            timeout: 30,
            encoding: "utf8".to_string(),
        }
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_encoding(mut self, encoding: String) -> Self {
        self.encoding = encoding;
        self
    }
}

impl Default for TCPConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            host: "127.0.0.1".to_string(),
            port: 8888,
            timeout: 30,
            encoding: "utf8".to_string(),
        }
    }
}

/// UDP configuration for a single instance
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UDPConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub host: String,
    pub port: u16,
    pub timeout: u64,
    pub encoding: String,
    pub broadcast: bool,
}

impl UDPConfig {
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        host: String,
        port: u16,
    ) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            host,
            port,
            timeout: 30,
            encoding: "utf8".to_string(),
            broadcast: false,
        }
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_encoding(mut self, encoding: String) -> Self {
        self.encoding = encoding;
        self
    }

    pub fn with_broadcast(mut self, broadcast: bool) -> Self {
        self.broadcast = broadcast;
        self
    }
}

impl Default for UDPConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            host: "127.0.0.1".to_string(),
            port: 9999,
            timeout: 30,
            encoding: "utf8".to_string(),
            broadcast: false,
        }
    }
}

/// FTP configuration for a single instance
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FTPConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub remote_dir: String,
    pub timeout: u64,
    pub mode: String,
}

impl FTPConfig {
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        host: String,
        port: u16,
    ) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            host,
            port,
            username: "anonymous".to_string(),
            password: String::new(),
            remote_dir: "/".to_string(),
            timeout: 30,
            mode: "binary".to_string(),
        }
    }

    pub fn with_credentials(mut self, username: String, password: String) -> Self {
        self.username = username;
        self.password = password;
        self
    }

    pub fn with_remote_dir(mut self, remote_dir: String) -> Self {
        self.remote_dir = remote_dir;
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_mode(mut self, mode: String) -> Self {
        self.mode = mode;
        self
    }
}

impl Default for FTPConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            host: String::new(),
            port: 21,
            username: "anonymous".to_string(),
            password: String::new(),
            remote_dir: "/".to_string(),
            timeout: 30,
            mode: "binary".to_string(),
        }
    }
}

/// SMTP configuration for a single instance
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SMTPConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
}

impl SMTPConfig {
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        host: String,
        port: u16,
        from: String,
    ) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            host,
            port,
            username: String::new(),
            password: String::new(),
            from,
        }
    }

    pub fn with_credentials(mut self, username: String, password: String) -> Self {
        self.username = username;
        self.password = password;
        self
    }
}

impl Default for SMTPConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            host: String::new(),
            port: 587,
            username: String::new(),
            password: String::new(),
            from: String::new(),
        }
    }
}

/// Telegram configuration for a single bot
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TelegramConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub bot_token: String,
}

impl TelegramConfig {
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        bot_token: String,
    ) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            bot_token,
        }
    }
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            bot_token: String::new(),
        }
    }
}

/// DingTalk configuration for a single robot
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DingTalkConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub access_token: String,
    pub secret: Option<String>,
}

impl DingTalkConfig {
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        access_token: String,
    ) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            access_token,
            secret: None,
        }
    }

    pub fn with_secret(mut self, secret: String) -> Self {
        self.secret = Some(secret);
        self
    }
}

impl Default for DingTalkConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            access_token: String::new(),
            secret: None,
        }
    }
}

/// Feishu configuration for a single webhook
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FeishuConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub webhook: String,
    pub secret: Option<String>,
}

impl FeishuConfig {
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        webhook: String,
    ) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            webhook,
            secret: None,
        }
    }

    pub fn with_secret(mut self, secret: String) -> Self {
        self.secret = Some(secret);
        self
    }
}

impl Default for FeishuConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            webhook: String::new(),
            secret: None,
        }
    }
}

/// WeCom configuration for a single webhook
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct WeComConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub webhook: String,
    pub key: Option<String>,
}

impl WeComConfig {
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        webhook: String,
    ) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            webhook,
            key: None,
        }
    }

    pub fn with_key(mut self, key: String) -> Self {
        self.key = Some(key);
        self
    }
}

impl Default for WeComConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            webhook: String::new(),
            key: None,
        }
    }
}

/// GitHub configuration for a single instance
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct GitHubConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub token: String,
    pub api_url: String,
    pub timeout: u64,
}

impl GitHubConfig {
    pub fn new(
        id: String,
        name: Option<String>,
        description: Option<String>,
        token: String,
    ) -> Self {
        let name = name.unwrap_or_else(|| id.clone());
        let description = description.unwrap_or_default();
        Self {
            id,
            name,
            description,
            token,
            api_url: "https://api.github.com".to_string(),
            timeout: 30,
        }
    }

    pub fn with_api_url(mut self, api_url: String) -> Self {
        self.api_url = api_url;
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            token: String::new(),
            api_url: "https://api.github.com".to_string(),
            timeout: 30,
        }
    }
}

/// Hippox global configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct HippoxConfig {
    // Application settings
    pub lang: String,

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

    /// Check if any PostgreSQL instances are configured
    pub fn has_postgresql(&self) -> bool {
        !self.postgresql_instances.is_empty()
    }

    /// Check if any MySQL instances are configured
    pub fn has_mysql(&self) -> bool {
        !self.mysql_instances.is_empty()
    }

    /// Check if any Redis instances are configured
    pub fn has_redis(&self) -> bool {
        !self.redis_instances.is_empty()
    }

    /// Check if any SQLite instances are configured
    pub fn has_sqlite(&self) -> bool {
        !self.sqlite_instances.is_empty()
    }

    /// Check if any Docker instances are configured
    pub fn has_docker(&self) -> bool {
        !self.docker_instances.is_empty()
    }

    /// Check if any Kubernetes instances are configured
    pub fn has_k8s(&self) -> bool {
        !self.k8s_instances.is_empty()
    }

    /// Check if any TCP instances are configured
    pub fn has_tcp(&self) -> bool {
        !self.tcp_instances.is_empty()
    }

    /// Check if any UDP instances are configured
    pub fn has_udp(&self) -> bool {
        !self.udp_instances.is_empty()
    }

    /// Check if any FTP instances are configured
    pub fn has_ftp(&self) -> bool {
        !self.ftp_instances.is_empty()
    }

    /// Check if any SMTP instances are configured
    pub fn has_smtp(&self) -> bool {
        !self.smtp_instances.is_empty()
    }

    /// Check if any Telegram instances are configured
    pub fn has_telegram(&self) -> bool {
        !self.telegram_instances.is_empty()
    }

    /// Check if any DingTalk instances are configured
    pub fn has_dingtalk(&self) -> bool {
        !self.dingtalk_instances.is_empty()
    }

    /// Check if any Feishu instances are configured
    pub fn has_feishu(&self) -> bool {
        !self.feishu_instances.is_empty()
    }

    /// Check if any WeCom instances are configured
    pub fn has_wecom(&self) -> bool {
        !self.wecom_instances.is_empty()
    }

    /// Check if any GitHub instances are configured
    pub fn has_github(&self) -> bool {
        !self.github_instances.is_empty()
    }
}

/// init global configuration from TOML file
pub(crate) fn init_config_from_toml_file(path: &str) -> anyhow::Result<()> {
    let config = HippoxConfig::load_from_toml_file(path)?;
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    *global = config;
    drop(global);
    Ok(())
}

/// init global configuration from JSON file
pub(crate) fn init_config_from_json_file(path: &str) -> anyhow::Result<()> {
    let config = HippoxConfig::load_from_json_file(path)?;
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    *global = config;
    drop(global);
    Ok(())
}

/// Initialize global configuration from a JSON string of optional parameters.
///
/// This function allows loading multiple instances of each service type from a JSON string.
/// Each instance requires an ID as the map key, and the configuration object contains
/// the specific settings for that instance.
///
/// # JSON Format Example
///
/// ```json
/// {
///   "lang": "zh",
///   "postgresql_instances": {
///     "pg_prod": {
///       "name": "Production PostgreSQL",
///       "description": "Main production database for the application",
///       "host": "localhost",
///       "port": 5432,
///       "database": "app_db",
///       "username": "user",
///       "password": "pass",
///       "pool_size": 20,
///       "timeout": 60
///     }
///   },
///   "redis_instances": {
///     "cache_main": {
///       "name": "Main Cache",
///       "description": "Redis cache for session storage and rate limiting",
///       "host": "localhost",
///       "port": 6379,
///       "password": "secret",
///       "db": 0
///     }
///   },
///   "docker_instances": {
///     "docker_local": {
///       "name": "Local Docker",
///       "description": "Local Docker daemon for container management",
///       "host": "unix:///var/run/docker.sock"
///     }
///   }
/// }
/// ```
///
/// # Supported Configuration Sections
///
/// - `lang`: Application language (string)
/// - `postgresql_instances`: Map of PostgreSQL instance configurations
/// - `mysql_instances`: Map of MySQL instance configurations
/// - `redis_instances`: Map of Redis instance configurations
/// - `sqlite_instances`: Map of SQLite instance configurations
/// - `docker_instances`: Map of Docker instance configurations
/// - `k8s_instances`: Map of Kubernetes cluster configurations
/// - `tcp_instances`: Map of TCP server configurations
/// - `udp_instances`: Map of UDP server configurations
/// - `ftp_instances`: Map of FTP server configurations
/// - `smtp_instances`: Map of SMTP server configurations
/// - `telegram_instances`: Map of Telegram bot configurations
/// - `dingtalk_instances`: Map of DingTalk robot configurations
/// - `feishu_instances`: Map of Feishu webhook configurations
/// - `wecom_instances`: Map of WeCom webhook configurations
/// - `github_instances`: Map of GitHub API configurations
///
/// # Returns
///
/// Returns `Ok(())` if the configuration is successfully loaded and applied,
/// or an error if the JSON string is invalid.
pub(crate) fn init_config_from_params_json_str(json_str: &str) -> anyhow::Result<()> {
    let overrides: serde_json::Value = serde_json::from_str(json_str)?;
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();

    // Lang
    if let Some(v) = overrides.get("lang").and_then(|x| x.as_str()) {
        global.lang = v.to_string();
    }

    // PostgreSQL instances
    if let Some(instances) = overrides
        .get("postgresql_instances")
        .and_then(|x| x.as_object())
    {
        for (id, config) in instances {
            if let (Some(host), Some(port), Some(database), Some(username), Some(password)) = (
                config.get("host").and_then(|x| x.as_str()),
                config.get("port").and_then(|x| x.as_u64()),
                config.get("database").and_then(|x| x.as_str()),
                config.get("username").and_then(|x| x.as_str()),
                config.get("password").and_then(|x| x.as_str()),
            ) {
                let name = config
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let description = config
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let pg_config = PostgreSQLConfig::new(
                    id.clone(),
                    name,
                    description,
                    host.to_string(),
                    port as u16,
                    database.to_string(),
                    username.to_string(),
                    password.to_string(),
                )
                .with_pool_size(
                    config
                        .get("pool_size")
                        .and_then(|x| x.as_u64())
                        .unwrap_or(10) as usize,
                )
                .with_timeout(config.get("timeout").and_then(|x| x.as_u64()).unwrap_or(30));
                global.add_postgresql_instance(pg_config);
            }
        }
    }

    // MySQL instances
    if let Some(instances) = overrides.get("mysql_instances").and_then(|x| x.as_object()) {
        for (id, config) in instances {
            if let (Some(host), Some(port), Some(database), Some(username), Some(password)) = (
                config.get("host").and_then(|x| x.as_str()),
                config.get("port").and_then(|x| x.as_u64()),
                config.get("database").and_then(|x| x.as_str()),
                config.get("username").and_then(|x| x.as_str()),
                config.get("password").and_then(|x| x.as_str()),
            ) {
                let name = config
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let description = config
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let mysql_config = MySQLConfig::new(
                    id.clone(),
                    name,
                    description,
                    host.to_string(),
                    port as u16,
                    database.to_string(),
                    username.to_string(),
                    password.to_string(),
                )
                .with_pool_size(
                    config
                        .get("pool_size")
                        .and_then(|x| x.as_u64())
                        .unwrap_or(10) as usize,
                )
                .with_timeout(config.get("timeout").and_then(|x| x.as_u64()).unwrap_or(30));
                global.add_mysql_instance(mysql_config);
            }
        }
    }

    // Redis instances
    if let Some(instances) = overrides.get("redis_instances").and_then(|x| x.as_object()) {
        for (id, config) in instances {
            if let (Some(host), Some(port)) = (
                config.get("host").and_then(|x| x.as_str()),
                config.get("port").and_then(|x| x.as_u64()),
            ) {
                let name = config
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let description = config
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let mut redis_config =
                    RedisConfig::new(id.clone(), name, description, host.to_string(), port as u16);
                if let Some(password) = config.get("password").and_then(|x| x.as_str()) {
                    redis_config = redis_config.with_password(password.to_string());
                }
                if let Some(db) = config.get("db").and_then(|x| x.as_u64()) {
                    redis_config = redis_config.with_db(db as usize);
                }
                redis_config = redis_config
                    .with_pool_size(
                        config
                            .get("pool_size")
                            .and_then(|x| x.as_u64())
                            .unwrap_or(10) as usize,
                    )
                    .with_timeout(config.get("timeout").and_then(|x| x.as_u64()).unwrap_or(30));
                global.add_redis_instance(redis_config);
            }
        }
    }

    // SQLite instances
    if let Some(instances) = overrides
        .get("sqlite_instances")
        .and_then(|x| x.as_object())
    {
        for (id, config) in instances {
            if let Some(path) = config.get("path").and_then(|x| x.as_str()) {
                let name = config
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let description = config
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let sqlite_config =
                    SQLiteConfig::new(id.clone(), name, description, path.to_string())
                        .with_pool_size(
                            config
                                .get("pool_size")
                                .and_then(|x| x.as_u64())
                                .unwrap_or(5) as usize,
                        )
                        .with_timeout(config.get("timeout").and_then(|x| x.as_u64()).unwrap_or(30));
                global.add_sqlite_instance(sqlite_config);
            }
        }
    }

    // Docker instances
    if let Some(instances) = overrides
        .get("docker_instances")
        .and_then(|x| x.as_object())
    {
        for (id, config) in instances {
            if let Some(host) = config.get("host").and_then(|x| x.as_str()) {
                let name = config
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let description = config
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let mut docker_config =
                    DockerConfig::new(id.clone(), name, description, host.to_string());
                if let Some(api_version) = config.get("api_version").and_then(|x| x.as_str()) {
                    docker_config = docker_config.with_api_version(api_version.to_string());
                }
                if let Some(timeout) = config.get("timeout").and_then(|x| x.as_u64()) {
                    docker_config = docker_config.with_timeout(timeout);
                }
                if let (Some(verify), Some(cert_path)) = (
                    config.get("tls_verify").and_then(|x| x.as_bool()),
                    config.get("cert_path").and_then(|x| x.as_str()),
                ) {
                    docker_config = docker_config.with_tls(verify, cert_path.to_string());
                }
                global.add_docker_instance(docker_config);
            }
        }
    }

    // Kubernetes instances
    if let Some(instances) = overrides.get("k8s_instances").and_then(|x| x.as_object()) {
        for (id, config) in instances {
            let name = config
                .get("name")
                .and_then(|x| x.as_str())
                .map(String::from);
            let description = config
                .get("description")
                .and_then(|x| x.as_str())
                .map(String::from);
            let mut k8s_config = K8sConfig::new(id.clone(), name, description);
            if let Some(kubeconfig) = config.get("kubeconfig").and_then(|x| x.as_str()) {
                k8s_config = k8s_config.with_kubeconfig(kubeconfig.to_string());
            }
            if let Some(context) = config.get("context").and_then(|x| x.as_str()) {
                k8s_config = k8s_config.with_context(context.to_string());
            }
            if let Some(namespace) = config.get("namespace").and_then(|x| x.as_str()) {
                k8s_config = k8s_config.with_namespace(namespace.to_string());
            }
            if let (Some(api_server), Some(token)) = (
                config.get("api_server").and_then(|x| x.as_str()),
                config.get("api_token").and_then(|x| x.as_str()),
            ) {
                k8s_config = k8s_config.with_api_server(api_server.to_string(), token.to_string());
            }
            if let Some(timeout) = config.get("timeout").and_then(|x| x.as_u64()) {
                k8s_config = k8s_config.with_timeout(timeout);
            }
            if let Some(insecure) = config.get("insecure").and_then(|x| x.as_bool()) {
                k8s_config = k8s_config.with_insecure(insecure);
            }
            if let Some(ca_cert) = config.get("ca_cert").and_then(|x| x.as_str()) {
                k8s_config = k8s_config.with_ca_cert(ca_cert.to_string());
            }
            if let (Some(client_cert), Some(client_key)) = (
                config.get("client_cert").and_then(|x| x.as_str()),
                config.get("client_key").and_then(|x| x.as_str()),
            ) {
                k8s_config =
                    k8s_config.with_client_cert(client_cert.to_string(), client_key.to_string());
            }
            global.add_k8s_instance(k8s_config);
        }
    }

    // TCP instances
    if let Some(instances) = overrides.get("tcp_instances").and_then(|x| x.as_object()) {
        for (id, config) in instances {
            if let (Some(host), Some(port)) = (
                config.get("host").and_then(|x| x.as_str()),
                config.get("port").and_then(|x| x.as_u64()),
            ) {
                let name = config
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let description = config
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let mut tcp_config =
                    TCPConfig::new(id.clone(), name, description, host.to_string(), port as u16);
                if let Some(timeout) = config.get("timeout").and_then(|x| x.as_u64()) {
                    tcp_config = tcp_config.with_timeout(timeout);
                }
                if let Some(encoding) = config.get("encoding").and_then(|x| x.as_str()) {
                    tcp_config = tcp_config.with_encoding(encoding.to_string());
                }
                global.add_tcp_instance(tcp_config);
            }
        }
    }

    // UDP instances
    if let Some(instances) = overrides.get("udp_instances").and_then(|x| x.as_object()) {
        for (id, config) in instances {
            if let (Some(host), Some(port)) = (
                config.get("host").and_then(|x| x.as_str()),
                config.get("port").and_then(|x| x.as_u64()),
            ) {
                let name = config
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let description = config
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let mut udp_config =
                    UDPConfig::new(id.clone(), name, description, host.to_string(), port as u16);
                if let Some(timeout) = config.get("timeout").and_then(|x| x.as_u64()) {
                    udp_config = udp_config.with_timeout(timeout);
                }
                if let Some(encoding) = config.get("encoding").and_then(|x| x.as_str()) {
                    udp_config = udp_config.with_encoding(encoding.to_string());
                }
                if let Some(broadcast) = config.get("broadcast").and_then(|x| x.as_bool()) {
                    udp_config = udp_config.with_broadcast(broadcast);
                }
                global.add_udp_instance(udp_config);
            }
        }
    }

    // FTP instances
    if let Some(instances) = overrides.get("ftp_instances").and_then(|x| x.as_object()) {
        for (id, config) in instances {
            if let (Some(host), Some(port)) = (
                config.get("host").and_then(|x| x.as_str()),
                config.get("port").and_then(|x| x.as_u64()),
            ) {
                let name = config
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let description = config
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let mut ftp_config =
                    FTPConfig::new(id.clone(), name, description, host.to_string(), port as u16);
                if let (Some(username), Some(password)) = (
                    config.get("username").and_then(|x| x.as_str()),
                    config.get("password").and_then(|x| x.as_str()),
                ) {
                    ftp_config =
                        ftp_config.with_credentials(username.to_string(), password.to_string());
                }
                if let Some(remote_dir) = config.get("remote_dir").and_then(|x| x.as_str()) {
                    ftp_config = ftp_config.with_remote_dir(remote_dir.to_string());
                }
                if let Some(timeout) = config.get("timeout").and_then(|x| x.as_u64()) {
                    ftp_config = ftp_config.with_timeout(timeout);
                }
                if let Some(mode) = config.get("mode").and_then(|x| x.as_str()) {
                    ftp_config = ftp_config.with_mode(mode.to_string());
                }
                global.add_ftp_instance(ftp_config);
            }
        }
    }

    // SMTP instances
    if let Some(instances) = overrides.get("smtp_instances").and_then(|x| x.as_object()) {
        for (id, config) in instances {
            if let (Some(host), Some(port), Some(from)) = (
                config.get("host").and_then(|x| x.as_str()),
                config.get("port").and_then(|x| x.as_u64()),
                config.get("from").and_then(|x| x.as_str()),
            ) {
                let name = config
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let description = config
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let mut smtp_config = SMTPConfig::new(
                    id.clone(),
                    name,
                    description,
                    host.to_string(),
                    port as u16,
                    from.to_string(),
                );
                if let (Some(username), Some(password)) = (
                    config.get("username").and_then(|x| x.as_str()),
                    config.get("password").and_then(|x| x.as_str()),
                ) {
                    smtp_config =
                        smtp_config.with_credentials(username.to_string(), password.to_string());
                }
                global.add_smtp_instance(smtp_config);
            }
        }
    }

    // Telegram instances
    if let Some(instances) = overrides
        .get("telegram_instances")
        .and_then(|x| x.as_object())
    {
        for (id, config) in instances {
            if let Some(bot_token) = config.get("bot_token").and_then(|x| x.as_str()) {
                let name = config
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let description = config
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let telegram_config =
                    TelegramConfig::new(id.clone(), name, description, bot_token.to_string());
                global.add_telegram_instance(telegram_config);
            }
        }
    }

    // DingTalk instances
    if let Some(instances) = overrides
        .get("dingtalk_instances")
        .and_then(|x| x.as_object())
    {
        for (id, config) in instances {
            if let Some(access_token) = config.get("access_token").and_then(|x| x.as_str()) {
                let name = config
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let description = config
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let mut dingtalk_config =
                    DingTalkConfig::new(id.clone(), name, description, access_token.to_string());
                if let Some(secret) = config.get("secret").and_then(|x| x.as_str()) {
                    dingtalk_config = dingtalk_config.with_secret(secret.to_string());
                }
                global.add_dingtalk_instance(dingtalk_config);
            }
        }
    }

    // Feishu instances
    if let Some(instances) = overrides
        .get("feishu_instances")
        .and_then(|x| x.as_object())
    {
        for (id, config) in instances {
            if let Some(webhook) = config.get("webhook").and_then(|x| x.as_str()) {
                let name = config
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let description = config
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let mut feishu_config =
                    FeishuConfig::new(id.clone(), name, description, webhook.to_string());
                if let Some(secret) = config.get("secret").and_then(|x| x.as_str()) {
                    feishu_config = feishu_config.with_secret(secret.to_string());
                }
                global.add_feishu_instance(feishu_config);
            }
        }
    }

    // WeCom instances
    if let Some(instances) = overrides.get("wecom_instances").and_then(|x| x.as_object()) {
        for (id, config) in instances {
            if let Some(webhook) = config.get("webhook").and_then(|x| x.as_str()) {
                let name = config
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let description = config
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let mut wecom_config =
                    WeComConfig::new(id.clone(), name, description, webhook.to_string());
                if let Some(key) = config.get("key").and_then(|x| x.as_str()) {
                    wecom_config = wecom_config.with_key(key.to_string());
                }
                global.add_wecom_instance(wecom_config);
            }
        }
    }

    // GitHub instances
    if let Some(instances) = overrides
        .get("github_instances")
        .and_then(|x| x.as_object())
    {
        for (id, config) in instances {
            if let Some(token) = config.get("token").and_then(|x| x.as_str()) {
                let name = config
                    .get("name")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let description = config
                    .get("description")
                    .and_then(|x| x.as_str())
                    .map(String::from);
                let mut github_config =
                    GitHubConfig::new(id.clone(), name, description, token.to_string());
                if let Some(api_url) = config.get("api_url").and_then(|x| x.as_str()) {
                    github_config = github_config.with_api_url(api_url.to_string());
                }
                if let Some(timeout) = config.get("timeout").and_then(|x| x.as_u64()) {
                    github_config = github_config.with_timeout(timeout);
                }
                global.add_github_instance(github_config);
            }
        }
    }

    drop(global);
    Ok(())
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
    drop(global);
    Ok(())
}

pub fn get_lang() -> String {
    HIPPOX_CORE_CONFIG.read().unwrap().lang.clone()
}

pub fn set_lang(lang: String) -> anyhow::Result<()> {
    let mut config = HIPPOX_CORE_CONFIG.write().unwrap();
    config.lang = lang;
    drop(config);
    Ok(())
}

pub fn add_postgresql_instance(config: PostgreSQLConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_postgresql_instance(config);
    drop(global);
    id
}

pub fn get_postgresql_instance(id: &str) -> Option<PostgreSQLConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_postgresql_instance(id).cloned()
}

pub fn remove_postgresql_instance(id: &str) -> Option<PostgreSQLConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_postgresql_instance(id);
    drop(global);
    result
}

pub fn list_postgresql_instances() -> Vec<PostgreSQLConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global
        .list_postgresql_instances()
        .into_iter()
        .cloned()
        .collect()
}

pub fn add_mysql_instance(config: MySQLConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_mysql_instance(config);
    drop(global);
    id
}

pub fn get_mysql_instance(id: &str) -> Option<MySQLConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_mysql_instance(id).cloned()
}

pub fn remove_mysql_instance(id: &str) -> Option<MySQLConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_mysql_instance(id);
    drop(global);
    result
}

pub fn list_mysql_instances() -> Vec<MySQLConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_mysql_instances().into_iter().cloned().collect()
}

pub fn add_redis_instance(config: RedisConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_redis_instance(config);
    drop(global);
    id
}

pub fn get_redis_instance(id: &str) -> Option<RedisConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_redis_instance(id).cloned()
}

pub fn remove_redis_instance(id: &str) -> Option<RedisConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_redis_instance(id);
    drop(global);
    result
}

pub fn list_redis_instances() -> Vec<RedisConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_redis_instances().into_iter().cloned().collect()
}

pub fn add_sqlite_instance(config: SQLiteConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_sqlite_instance(config);
    drop(global);
    id
}

pub fn get_sqlite_instance(id: &str) -> Option<SQLiteConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_sqlite_instance(id).cloned()
}

pub fn remove_sqlite_instance(id: &str) -> Option<SQLiteConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_sqlite_instance(id);
    drop(global);
    result
}

pub fn list_sqlite_instances() -> Vec<SQLiteConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global
        .list_sqlite_instances()
        .into_iter()
        .cloned()
        .collect()
}

pub fn add_docker_instance(config: DockerConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_docker_instance(config);
    drop(global);
    id
}

pub fn get_docker_instance(id: &str) -> Option<DockerConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_docker_instance(id).cloned()
}

pub fn remove_docker_instance(id: &str) -> Option<DockerConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_docker_instance(id);
    drop(global);
    result
}

pub fn list_docker_instances() -> Vec<DockerConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global
        .list_docker_instances()
        .into_iter()
        .cloned()
        .collect()
}

pub fn add_k8s_instance(config: K8sConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_k8s_instance(config);
    drop(global);
    id
}

pub fn get_k8s_instance(id: &str) -> Option<K8sConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_k8s_instance(id).cloned()
}

pub fn remove_k8s_instance(id: &str) -> Option<K8sConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_k8s_instance(id);
    drop(global);
    result
}

pub fn list_k8s_instances() -> Vec<K8sConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_k8s_instances().into_iter().cloned().collect()
}

pub fn add_tcp_instance(config: TCPConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_tcp_instance(config);
    drop(global);
    id
}

pub fn get_tcp_instance(id: &str) -> Option<TCPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_tcp_instance(id).cloned()
}

pub fn remove_tcp_instance(id: &str) -> Option<TCPConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_tcp_instance(id);
    drop(global);
    result
}

pub fn list_tcp_instances() -> Vec<TCPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_tcp_instances().into_iter().cloned().collect()
}

pub fn add_udp_instance(config: UDPConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_udp_instance(config);
    drop(global);
    id
}

pub fn get_udp_instance(id: &str) -> Option<UDPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_udp_instance(id).cloned()
}

pub fn remove_udp_instance(id: &str) -> Option<UDPConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_udp_instance(id);
    drop(global);
    result
}

pub fn list_udp_instances() -> Vec<UDPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_udp_instances().into_iter().cloned().collect()
}

pub fn add_ftp_instance(config: FTPConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_ftp_instance(config);
    drop(global);
    id
}

pub fn get_ftp_instance(id: &str) -> Option<FTPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_ftp_instance(id).cloned()
}

pub fn remove_ftp_instance(id: &str) -> Option<FTPConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_ftp_instance(id);
    drop(global);
    result
}

pub fn list_ftp_instances() -> Vec<FTPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_ftp_instances().into_iter().cloned().collect()
}

pub fn add_smtp_instance(config: SMTPConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_smtp_instance(config);
    drop(global);
    id
}

pub fn get_smtp_instance(id: &str) -> Option<SMTPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_smtp_instance(id).cloned()
}

pub fn remove_smtp_instance(id: &str) -> Option<SMTPConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_smtp_instance(id);
    drop(global);
    result
}

pub fn list_smtp_instances() -> Vec<SMTPConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_smtp_instances().into_iter().cloned().collect()
}

pub fn add_telegram_instance(config: TelegramConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_telegram_instance(config);
    drop(global);
    id
}

pub fn get_telegram_instance(id: &str) -> Option<TelegramConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_telegram_instance(id).cloned()
}

pub fn remove_telegram_instance(id: &str) -> Option<TelegramConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_telegram_instance(id);
    drop(global);
    result
}

pub fn list_telegram_instances() -> Vec<TelegramConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global
        .list_telegram_instances()
        .into_iter()
        .cloned()
        .collect()
}

pub fn add_dingtalk_instance(config: DingTalkConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_dingtalk_instance(config);
    drop(global);
    id
}

pub fn get_dingtalk_instance(id: &str) -> Option<DingTalkConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_dingtalk_instance(id).cloned()
}

pub fn remove_dingtalk_instance(id: &str) -> Option<DingTalkConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_dingtalk_instance(id);
    drop(global);
    result
}

pub fn list_dingtalk_instances() -> Vec<DingTalkConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global
        .list_dingtalk_instances()
        .into_iter()
        .cloned()
        .collect()
}

pub fn add_feishu_instance(config: FeishuConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_feishu_instance(config);
    drop(global);
    id
}

pub fn get_feishu_instance(id: &str) -> Option<FeishuConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_feishu_instance(id).cloned()
}

pub fn remove_feishu_instance(id: &str) -> Option<FeishuConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_feishu_instance(id);
    drop(global);
    result
}

pub fn list_feishu_instances() -> Vec<FeishuConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global
        .list_feishu_instances()
        .into_iter()
        .cloned()
        .collect()
}

pub fn add_wecom_instance(config: WeComConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_wecom_instance(config);
    drop(global);
    id
}

pub fn get_wecom_instance(id: &str) -> Option<WeComConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_wecom_instance(id).cloned()
}

pub fn remove_wecom_instance(id: &str) -> Option<WeComConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_wecom_instance(id);
    drop(global);
    result
}

pub fn list_wecom_instances() -> Vec<WeComConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.list_wecom_instances().into_iter().cloned().collect()
}

pub fn add_github_instance(config: GitHubConfig) -> String {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let id = global.add_github_instance(config);
    drop(global);
    id
}

pub fn get_github_instance(id: &str) -> Option<GitHubConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_github_instance(id).cloned()
}

pub fn remove_github_instance(id: &str) -> Option<GitHubConfig> {
    let mut global = HIPPOX_CORE_CONFIG.write().unwrap();
    let result = global.remove_github_instance(id);
    drop(global);
    result
}

pub fn list_github_instances() -> Vec<GitHubConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global
        .list_github_instances()
        .into_iter()
        .cloned()
        .collect()
}

// ... 现有代码 ...

// Helper functions for getting instance configs by ID
pub fn get_postgresql_instance_by_id(id: &str) -> Option<PostgreSQLConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_postgresql_instance(id).cloned()
}

pub fn get_mysql_instance_by_id(id: &str) -> Option<MySQLConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_mysql_instance(id).cloned()
}

pub fn get_redis_instance_by_id(id: &str) -> Option<RedisConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_redis_instance(id).cloned()
}

pub fn get_sqlite_instance_by_id(id: &str) -> Option<SQLiteConfig> {
    let global = HIPPOX_CORE_CONFIG.read().unwrap();
    global.get_sqlite_instance(id).cloned()
}
