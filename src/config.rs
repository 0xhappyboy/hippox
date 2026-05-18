use once_cell::sync::Lazy;
use std::sync::RwLock;

use crate::envs;

/// Global static configuration instance
pub static GLOBAL_CONFIG: Lazy<RwLock<HippoxConfig>> =
    Lazy::new(|| RwLock::new(HippoxConfig::default()));

/// Hippox global configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct HippoxConfig {
    // Application settings
    pub lang: String,
    // SMTP settings
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from: String,
    // Telegram settings
    pub telegram_bot_token: String,
    // DingTalk settings
    pub dingding_access_token: String,
    // Feishu settings
    pub feishu_webhook: String,
    // WeCom settings
    pub wecom_webhook: String,
    // ==================== FTP Settings ====================
    pub ftp_host: String,
    pub ftp_port: u16,
    pub ftp_username: String,
    pub ftp_password: String,
    pub ftp_remote_dir: String,
    pub ftp_timeout: u64,
    pub ftp_mode: String,
    // ==================== TCP Settings ====================
    pub tcp_host: String,
    pub tcp_port: u16,
    pub tcp_timeout: u64,
    pub tcp_encoding: String,
    // ==================== UDP Settings ====================
    pub udp_host: String,
    pub udp_port: u16,
    pub udp_timeout: u64,
    pub udp_encoding: String,
    pub udp_broadcast: bool,
    // ==================== PostgreSQL Settings ====================
    pub pg_host: String,
    pub pg_port: u16,
    pub pg_database: String,
    pub pg_username: String,
    pub pg_password: String,
    pub pg_pool_size: usize,
    pub pg_timeout: u64,
    // ==================== MySQL Settings ====================
    pub mysql_host: String,
    pub mysql_port: u16,
    pub mysql_database: String,
    pub mysql_username: String,
    pub mysql_password: String,
    pub mysql_pool_size: usize,
    pub mysql_timeout: u64,
    // ==================== Redis Settings ====================
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_password: String,
    pub redis_db: usize,
    pub redis_pool_size: usize,
    pub redis_timeout: u64,
    // ==================== SQLite Settings ====================
    pub sqlite_path: String,
    pub sqlite_pool_size: usize,
    pub sqlite_timeout: u64,
    // ==================== GitHub Settings ====================
    pub github_token: String,
    pub github_api_url: String,
    pub github_timeout: u64,
}

impl Default for HippoxConfig {
    fn default() -> Self {
        Self {
            lang: "en".to_string(),
            smtp_host: String::new(),
            smtp_port: 587,
            smtp_username: String::new(),
            smtp_password: String::new(),
            smtp_from: String::new(),
            telegram_bot_token: String::new(),
            dingding_access_token: String::new(),
            feishu_webhook: String::new(),
            wecom_webhook: String::new(),
            // FTP defaults
            ftp_host: String::new(),
            ftp_port: 21,
            ftp_username: "anonymous".to_string(),
            ftp_password: String::new(),
            ftp_remote_dir: "/".to_string(),
            ftp_timeout: 30,
            ftp_mode: "binary".to_string(),
            // TCP defaults
            tcp_host: "127.0.0.1".to_string(),
            tcp_port: 8888,
            tcp_timeout: 30,
            tcp_encoding: "utf8".to_string(),
            // UDP defaults
            udp_host: "127.0.0.1".to_string(),
            udp_port: 9999,
            udp_timeout: 30,
            udp_encoding: "utf8".to_string(),
            udp_broadcast: false,
            // PostgreSQL defaults
            pg_host: String::new(),
            pg_port: 5432,
            pg_database: String::new(),
            pg_username: String::new(),
            pg_password: String::new(),
            pg_pool_size: 10,
            pg_timeout: 30,
            // MySQL defaults
            mysql_host: String::new(),
            mysql_port: 3306,
            mysql_database: String::new(),
            mysql_username: String::new(),
            mysql_password: String::new(),
            mysql_pool_size: 10,
            mysql_timeout: 30,
            // Redis defaults
            redis_host: String::new(),
            redis_port: 6379,
            redis_password: String::new(),
            redis_db: 0,
            redis_pool_size: 10,
            redis_timeout: 30,
            // SQLite defaults
            sqlite_path: String::new(),
            sqlite_pool_size: 5,
            sqlite_timeout: 30,
            // GitHub defaults
            github_token: String::new(),
            github_api_url: "https://api.github.com".to_string(),
            github_timeout: 30,
        }
    }
}

impl HippoxConfig {
    /// Load configuration from environment variables
    pub fn load_from_env() -> Self {
        Self {
            lang: envs::get_env_or(envs::HIPPOX_LANG, "en"),
            smtp_host: envs::get_env_or(envs::HIPPOX_SMTP_HOST, ""),
            smtp_port: envs::get_env_or(envs::HIPPOX_SMTP_PORT, "587")
                .parse::<u16>()
                .unwrap_or(587),
            smtp_username: envs::get_env_or(envs::HIPPOX_SMTP_USERNAME, ""),
            smtp_password: envs::get_env_or(envs::HIPPOX_SMTP_PASSWORD, ""),
            smtp_from: envs::get_env_or(envs::HIPPOX_SMTP_FROM, ""),
            telegram_bot_token: envs::get_env_or(envs::HIPPOX_TELEGRAM_BOT_TOKEN, ""),
            dingding_access_token: envs::get_env_or(envs::HIPPOX_DINGDING_ACCESS_TOKEN, ""),
            feishu_webhook: envs::get_env_or(envs::HIPPOX_FEISHU_WEBHOOK, ""),
            wecom_webhook: envs::get_env_or(envs::HIPPOX_WECOM_WEBHOOK, ""),
            // FTP
            ftp_host: envs::get_env_or(envs::HIPPOX_FTP_HOST, ""),
            ftp_port: envs::get_env_or(envs::HIPPOX_FTP_PORT, "21")
                .parse()
                .unwrap_or(21),
            ftp_username: envs::get_env_or(envs::HIPPOX_FTP_USERNAME, "anonymous"),
            ftp_password: envs::get_env_or(envs::HIPPOX_FTP_PASSWORD, ""),
            ftp_remote_dir: envs::get_env_or(envs::HIPPOX_FTP_REMOTE_DIR, "/"),
            ftp_timeout: envs::get_env_or(envs::HIPPOX_FTP_TIMEOUT, "30")
                .parse()
                .unwrap_or(30),
            ftp_mode: envs::get_env_or(envs::HIPPOX_FTP_MODE, "binary"),
            // TCP
            tcp_host: envs::get_env_or(envs::HIPPOX_TCP_HOST, "127.0.0.1"),
            tcp_port: envs::get_env_or(envs::HIPPOX_TCP_PORT, "8888")
                .parse()
                .unwrap_or(8888),
            tcp_timeout: envs::get_env_or(envs::HIPPOX_TCP_TIMEOUT, "30")
                .parse()
                .unwrap_or(30),
            tcp_encoding: envs::get_env_or(envs::HIPPOX_TCP_ENCODING, "utf8"),
            // UDP
            udp_host: envs::get_env_or(envs::HIPPOX_UDP_HOST, "127.0.0.1"),
            udp_port: envs::get_env_or(envs::HIPPOX_UDP_PORT, "9999")
                .parse()
                .unwrap_or(9999),
            udp_timeout: envs::get_env_or(envs::HIPPOX_UDP_TIMEOUT, "30")
                .parse()
                .unwrap_or(30),
            udp_encoding: envs::get_env_or(envs::HIPPOX_UDP_ENCODING, "utf8"),
            udp_broadcast: envs::is_env_true(envs::HIPPOX_UDP_BROADCAST),
            // PostgreSQL
            pg_host: envs::get_env_or(envs::HIPPOX_PG_HOST, ""),
            pg_port: envs::get_env_or(envs::HIPPOX_PG_PORT, "5432")
                .parse()
                .unwrap_or(5432),
            pg_database: envs::get_env_or(envs::HIPPOX_PG_DATABASE, ""),
            pg_username: envs::get_env_or(envs::HIPPOX_PG_USERNAME, ""),
            pg_password: envs::get_env_or(envs::HIPPOX_PG_PASSWORD, ""),
            pg_pool_size: envs::get_env_or(envs::HIPPOX_PG_POOL_SIZE, "10")
                .parse()
                .unwrap_or(10),
            pg_timeout: envs::get_env_or(envs::HIPPOX_PG_TIMEOUT, "30")
                .parse()
                .unwrap_or(30),
            // MySQL
            mysql_host: envs::get_env_or(envs::HIPPOX_MYSQL_HOST, ""),
            mysql_port: envs::get_env_or(envs::HIPPOX_MYSQL_PORT, "3306")
                .parse()
                .unwrap_or(3306),
            mysql_database: envs::get_env_or(envs::HIPPOX_MYSQL_DATABASE, ""),
            mysql_username: envs::get_env_or(envs::HIPPOX_MYSQL_USERNAME, ""),
            mysql_password: envs::get_env_or(envs::HIPPOX_MYSQL_PASSWORD, ""),
            mysql_pool_size: envs::get_env_or(envs::HIPPOX_MYSQL_POOL_SIZE, "10")
                .parse()
                .unwrap_or(10),
            mysql_timeout: envs::get_env_or(envs::HIPPOX_MYSQL_TIMEOUT, "30")
                .parse()
                .unwrap_or(30),
            // Redis
            redis_host: envs::get_env_or(envs::HIPPOX_REDIS_HOST, ""),
            redis_port: envs::get_env_or(envs::HIPPOX_REDIS_PORT, "6379")
                .parse()
                .unwrap_or(6379),
            redis_password: envs::get_env_or(envs::HIPPOX_REDIS_PASSWORD, ""),
            redis_db: envs::get_env_or(envs::HIPPOX_REDIS_DB, "0")
                .parse()
                .unwrap_or(0),
            redis_pool_size: envs::get_env_or(envs::HIPPOX_REDIS_POOL_SIZE, "10")
                .parse()
                .unwrap_or(10),
            redis_timeout: envs::get_env_or(envs::HIPPOX_REDIS_TIMEOUT, "30")
                .parse()
                .unwrap_or(30),
            // SQLite
            sqlite_path: envs::get_env_or(envs::HIPPOX_SQLITE_PATH, ""),
            sqlite_pool_size: envs::get_env_or(envs::HIPPOX_SQLITE_POOL_SIZE, "5")
                .parse()
                .unwrap_or(5),
            sqlite_timeout: envs::get_env_or(envs::HIPPOX_SQLITE_TIMEOUT, "30")
                .parse()
                .unwrap_or(30),
            // GitHub
            github_token: envs::get_env_or(envs::HIPPOX_GITHUB_TOKEN, ""),
            github_api_url: envs::get_env_or(envs::HIPPOX_GITHUB_API_URL, "https://api.github.com"),
            github_timeout: envs::get_env_or(envs::HIPPOX_GITHUB_TIMEOUT, "30")
                .parse()
                .unwrap_or(30),
        }
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

    /// Load from optional parameters, only set fields that are Some
    #[allow(clippy::too_many_arguments)]
    pub fn load_from_params(
        lang: Option<String>,
        provider: Option<String>,
        enable_cli: Option<bool>,
        enable_tcp: Option<bool>,
        enable_http: Option<bool>,
        enable_websocket: Option<bool>,
        enable_grpc: Option<bool>,
        smtp_host: Option<String>,
        smtp_port: Option<u16>,
        smtp_username: Option<String>,
        smtp_password: Option<String>,
        smtp_from: Option<String>,
        telegram_bot_token: Option<String>,
        dingding_access_token: Option<String>,
        feishu_webhook: Option<String>,
        wecom_webhook: Option<String>,
        // FTP parameters
        ftp_host: Option<String>,
        ftp_port: Option<u16>,
        ftp_username: Option<String>,
        ftp_password: Option<String>,
        ftp_remote_dir: Option<String>,
        ftp_timeout: Option<u64>,
        ftp_mode: Option<String>,
        // TCP parameters
        tcp_host: Option<String>,
        tcp_port: Option<u16>,
        tcp_timeout: Option<u64>,
        tcp_encoding: Option<String>,
        // UDP parameters
        udp_host: Option<String>,
        udp_port: Option<u16>,
        udp_timeout: Option<u64>,
        udp_encoding: Option<String>,
        udp_broadcast: Option<bool>,
        // PostgreSQL parameters
        pg_host: Option<String>,
        pg_port: Option<u16>,
        pg_database: Option<String>,
        pg_username: Option<String>,
        pg_password: Option<String>,
        pg_pool_size: Option<usize>,
        pg_timeout: Option<u64>,
        // MySQL parameters
        mysql_host: Option<String>,
        mysql_port: Option<u16>,
        mysql_database: Option<String>,
        mysql_username: Option<String>,
        mysql_password: Option<String>,
        mysql_pool_size: Option<usize>,
        mysql_timeout: Option<u64>,
        // Redis parameters
        redis_host: Option<String>,
        redis_port: Option<u16>,
        redis_password: Option<String>,
        redis_db: Option<usize>,
        redis_pool_size: Option<usize>,
        redis_timeout: Option<u64>,
        // SQLite parameters
        sqlite_path: Option<String>,
        sqlite_pool_size: Option<usize>,
        sqlite_timeout: Option<u64>,
        // GitHub parameters
        github_token: Option<String>,
        github_api_url: Option<String>,
        github_timeout: Option<u64>,
    ) -> Self {
        let mut config = Self::load_from_env();
        if let Some(v) = lang {
            config.lang = v;
        }
        if let Some(v) = smtp_host {
            config.smtp_host = v;
        }
        if let Some(v) = smtp_port {
            config.smtp_port = v;
        }
        if let Some(v) = smtp_username {
            config.smtp_username = v;
        }
        if let Some(v) = smtp_password {
            config.smtp_password = v;
        }
        if let Some(v) = smtp_from {
            config.smtp_from = v;
        }
        if let Some(v) = telegram_bot_token {
            config.telegram_bot_token = v;
        }
        if let Some(v) = dingding_access_token {
            config.dingding_access_token = v;
        }
        if let Some(v) = feishu_webhook {
            config.feishu_webhook = v;
        }
        if let Some(v) = wecom_webhook {
            config.wecom_webhook = v;
        }
        // FTP
        if let Some(v) = ftp_host {
            config.ftp_host = v;
        }
        if let Some(v) = ftp_port {
            config.ftp_port = v;
        }
        if let Some(v) = ftp_username {
            config.ftp_username = v;
        }
        if let Some(v) = ftp_password {
            config.ftp_password = v;
        }
        if let Some(v) = ftp_remote_dir {
            config.ftp_remote_dir = v;
        }
        if let Some(v) = ftp_timeout {
            config.ftp_timeout = v;
        }
        if let Some(v) = ftp_mode {
            config.ftp_mode = v;
        }
        // TCP
        if let Some(v) = tcp_host {
            config.tcp_host = v;
        }
        if let Some(v) = tcp_port {
            config.tcp_port = v;
        }
        if let Some(v) = tcp_timeout {
            config.tcp_timeout = v;
        }
        if let Some(v) = tcp_encoding {
            config.tcp_encoding = v;
        }
        // UDP
        if let Some(v) = udp_host {
            config.udp_host = v;
        }
        if let Some(v) = udp_port {
            config.udp_port = v;
        }
        if let Some(v) = udp_timeout {
            config.udp_timeout = v;
        }
        if let Some(v) = udp_encoding {
            config.udp_encoding = v;
        }
        if let Some(v) = udp_broadcast {
            config.udp_broadcast = v;
        }
        // PostgreSQL
        if let Some(v) = pg_host {
            config.pg_host = v;
        }
        if let Some(v) = pg_port {
            config.pg_port = v;
        }
        if let Some(v) = pg_database {
            config.pg_database = v;
        }
        if let Some(v) = pg_username {
            config.pg_username = v;
        }
        if let Some(v) = pg_password {
            config.pg_password = v;
        }
        if let Some(v) = pg_pool_size {
            config.pg_pool_size = v;
        }
        if let Some(v) = pg_timeout {
            config.pg_timeout = v;
        }
        // MySQL
        if let Some(v) = mysql_host {
            config.mysql_host = v;
        }
        if let Some(v) = mysql_port {
            config.mysql_port = v;
        }
        if let Some(v) = mysql_database {
            config.mysql_database = v;
        }
        if let Some(v) = mysql_username {
            config.mysql_username = v;
        }
        if let Some(v) = mysql_password {
            config.mysql_password = v;
        }
        if let Some(v) = mysql_pool_size {
            config.mysql_pool_size = v;
        }
        if let Some(v) = mysql_timeout {
            config.mysql_timeout = v;
        }
        // Redis
        if let Some(v) = redis_host {
            config.redis_host = v;
        }
        if let Some(v) = redis_port {
            config.redis_port = v;
        }
        if let Some(v) = redis_password {
            config.redis_password = v;
        }
        if let Some(v) = redis_db {
            config.redis_db = v;
        }
        if let Some(v) = redis_pool_size {
            config.redis_pool_size = v;
        }
        if let Some(v) = redis_timeout {
            config.redis_timeout = v;
        }
        // SQLite
        if let Some(v) = sqlite_path {
            config.sqlite_path = v;
        }
        if let Some(v) = sqlite_pool_size {
            config.sqlite_pool_size = v;
        }
        if let Some(v) = sqlite_timeout {
            config.sqlite_timeout = v;
        }
        // GitHub
        if let Some(v) = github_token {
            config.github_token = v;
        }
        if let Some(v) = github_api_url {
            config.github_api_url = v;
        }
        if let Some(v) = github_timeout {
            config.github_timeout = v;
        }
        config
    }

    /// Load from JSON string of optional parameters, only set fields that are present
    pub fn load_from_params_json_str(json_str: &str) -> anyhow::Result<Self> {
        let overrides: serde_json::Value = serde_json::from_str(json_str)?;
        let mut config = Self::load_from_env();
        if let Some(v) = overrides.get("lang").and_then(|x| x.as_str()) {
            config.lang = v.to_string();
        }
        if let Some(v) = overrides.get("smtp_host").and_then(|x| x.as_str()) {
            config.smtp_host = v.to_string();
        }
        if let Some(v) = overrides.get("smtp_port").and_then(|x| x.as_u64()) {
            config.smtp_port = v as u16;
        }
        if let Some(v) = overrides.get("smtp_username").and_then(|x| x.as_str()) {
            config.smtp_username = v.to_string();
        }
        if let Some(v) = overrides.get("smtp_password").and_then(|x| x.as_str()) {
            config.smtp_password = v.to_string();
        }
        if let Some(v) = overrides.get("smtp_from").and_then(|x| x.as_str()) {
            config.smtp_from = v.to_string();
        }
        if let Some(v) = overrides.get("telegram_bot_token").and_then(|x| x.as_str()) {
            config.telegram_bot_token = v.to_string();
        }
        if let Some(v) = overrides
            .get("dingding_access_token")
            .and_then(|x| x.as_str())
        {
            config.dingding_access_token = v.to_string();
        }
        if let Some(v) = overrides.get("feishu_webhook").and_then(|x| x.as_str()) {
            config.feishu_webhook = v.to_string();
        }
        if let Some(v) = overrides.get("wecom_webhook").and_then(|x| x.as_str()) {
            config.wecom_webhook = v.to_string();
        }
        // FTP
        if let Some(v) = overrides.get("ftp_host").and_then(|x| x.as_str()) {
            config.ftp_host = v.to_string();
        }
        if let Some(v) = overrides.get("ftp_port").and_then(|x| x.as_u64()) {
            config.ftp_port = v as u16;
        }
        if let Some(v) = overrides.get("ftp_username").and_then(|x| x.as_str()) {
            config.ftp_username = v.to_string();
        }
        if let Some(v) = overrides.get("ftp_password").and_then(|x| x.as_str()) {
            config.ftp_password = v.to_string();
        }
        if let Some(v) = overrides.get("ftp_remote_dir").and_then(|x| x.as_str()) {
            config.ftp_remote_dir = v.to_string();
        }
        if let Some(v) = overrides.get("ftp_timeout").and_then(|x| x.as_u64()) {
            config.ftp_timeout = v;
        }
        if let Some(v) = overrides.get("ftp_mode").and_then(|x| x.as_str()) {
            config.ftp_mode = v.to_string();
        }
        // TCP
        if let Some(v) = overrides.get("tcp_host").and_then(|x| x.as_str()) {
            config.tcp_host = v.to_string();
        }
        if let Some(v) = overrides.get("tcp_port").and_then(|x| x.as_u64()) {
            config.tcp_port = v as u16;
        }
        if let Some(v) = overrides.get("tcp_timeout").and_then(|x| x.as_u64()) {
            config.tcp_timeout = v;
        }
        if let Some(v) = overrides.get("tcp_encoding").and_then(|x| x.as_str()) {
            config.tcp_encoding = v.to_string();
        }
        // UDP
        if let Some(v) = overrides.get("udp_host").and_then(|x| x.as_str()) {
            config.udp_host = v.to_string();
        }
        if let Some(v) = overrides.get("udp_port").and_then(|x| x.as_u64()) {
            config.udp_port = v as u16;
        }
        if let Some(v) = overrides.get("udp_timeout").and_then(|x| x.as_u64()) {
            config.udp_timeout = v;
        }
        if let Some(v) = overrides.get("udp_encoding").and_then(|x| x.as_str()) {
            config.udp_encoding = v.to_string();
        }
        if let Some(v) = overrides.get("udp_broadcast").and_then(|x| x.as_bool()) {
            config.udp_broadcast = v;
        }
        // PostgreSQL
        if let Some(v) = overrides.get("pg_host").and_then(|x| x.as_str()) {
            config.pg_host = v.to_string();
        }
        if let Some(v) = overrides.get("pg_port").and_then(|x| x.as_u64()) {
            config.pg_port = v as u16;
        }
        if let Some(v) = overrides.get("pg_database").and_then(|x| x.as_str()) {
            config.pg_database = v.to_string();
        }
        if let Some(v) = overrides.get("pg_username").and_then(|x| x.as_str()) {
            config.pg_username = v.to_string();
        }
        if let Some(v) = overrides.get("pg_password").and_then(|x| x.as_str()) {
            config.pg_password = v.to_string();
        }
        if let Some(v) = overrides.get("pg_pool_size").and_then(|x| x.as_u64()) {
            config.pg_pool_size = v as usize;
        }
        if let Some(v) = overrides.get("pg_timeout").and_then(|x| x.as_u64()) {
            config.pg_timeout = v;
        }
        // MySQL
        if let Some(v) = overrides.get("mysql_host").and_then(|x| x.as_str()) {
            config.mysql_host = v.to_string();
        }
        if let Some(v) = overrides.get("mysql_port").and_then(|x| x.as_u64()) {
            config.mysql_port = v as u16;
        }
        if let Some(v) = overrides.get("mysql_database").and_then(|x| x.as_str()) {
            config.mysql_database = v.to_string();
        }
        if let Some(v) = overrides.get("mysql_username").and_then(|x| x.as_str()) {
            config.mysql_username = v.to_string();
        }
        if let Some(v) = overrides.get("mysql_password").and_then(|x| x.as_str()) {
            config.mysql_password = v.to_string();
        }
        if let Some(v) = overrides.get("mysql_pool_size").and_then(|x| x.as_u64()) {
            config.mysql_pool_size = v as usize;
        }
        if let Some(v) = overrides.get("mysql_timeout").and_then(|x| x.as_u64()) {
            config.mysql_timeout = v;
        }
        // Redis
        if let Some(v) = overrides.get("redis_host").and_then(|x| x.as_str()) {
            config.redis_host = v.to_string();
        }
        if let Some(v) = overrides.get("redis_port").and_then(|x| x.as_u64()) {
            config.redis_port = v as u16;
        }
        if let Some(v) = overrides.get("redis_password").and_then(|x| x.as_str()) {
            config.redis_password = v.to_string();
        }
        if let Some(v) = overrides.get("redis_db").and_then(|x| x.as_u64()) {
            config.redis_db = v as usize;
        }
        if let Some(v) = overrides.get("redis_pool_size").and_then(|x| x.as_u64()) {
            config.redis_pool_size = v as usize;
        }
        if let Some(v) = overrides.get("redis_timeout").and_then(|x| x.as_u64()) {
            config.redis_timeout = v;
        }
        // SQLite
        if let Some(v) = overrides.get("sqlite_path").and_then(|x| x.as_str()) {
            config.sqlite_path = v.to_string();
        }
        if let Some(v) = overrides.get("sqlite_pool_size").and_then(|x| x.as_u64()) {
            config.sqlite_pool_size = v as usize;
        }
        if let Some(v) = overrides.get("sqlite_timeout").and_then(|x| x.as_u64()) {
            config.sqlite_timeout = v;
        }
        // GitHub
        if let Some(v) = overrides.get("github_token").and_then(|x| x.as_str()) {
            config.github_token = v.to_string();
        }
        if let Some(v) = overrides.get("github_api_url").and_then(|x| x.as_str()) {
            config.github_api_url = v.to_string();
        }
        if let Some(v) = overrides.get("github_timeout").and_then(|x| x.as_u64()) {
            config.github_timeout = v;
        }
        Ok(config)
    }

    pub fn is_github_configured(&self) -> bool {
        !self.github_token.is_empty()
    }

    /// Check if SMTP is configured
    pub fn is_smtp_configured(&self) -> bool {
        !self.smtp_host.is_empty()
            && !self.smtp_username.is_empty()
            && !self.smtp_password.is_empty()
            && !self.smtp_from.is_empty()
    }

    /// Check if Telegram is configured
    pub fn is_telegram_configured(&self) -> bool {
        !self.telegram_bot_token.is_empty()
    }

    /// Check if DingTalk is configured
    pub fn is_dingtalk_configured(&self) -> bool {
        !self.dingding_access_token.is_empty()
    }

    /// Check if Feishu is configured
    pub fn is_feishu_configured(&self) -> bool {
        !self.feishu_webhook.is_empty()
    }

    /// Check if WeCom is configured
    pub fn is_wecom_configured(&self) -> bool {
        !self.wecom_webhook.is_empty()
    }

    /// Check if FTP is configured
    pub fn is_ftp_configured(&self) -> bool {
        !self.ftp_host.is_empty()
    }

    /// Check if TCP is configured
    pub fn is_tcp_configured(&self) -> bool {
        self.tcp_port > 0
    }

    /// Check if UDP is configured
    pub fn is_udp_configured(&self) -> bool {
        self.udp_port > 0
    }

    pub fn is_postgresql_configured(&self) -> bool {
        !self.pg_host.is_empty() && !self.pg_database.is_empty()
    }

    pub fn is_mysql_configured(&self) -> bool {
        !self.mysql_host.is_empty() && !self.mysql_database.is_empty()
    }

    pub fn is_redis_configured(&self) -> bool {
        !self.redis_host.is_empty()
    }

    pub fn is_sqlite_configured(&self) -> bool {
        !self.sqlite_path.is_empty()
    }
}

/// init global configuration from environment variables
pub(crate) fn init_config_from_env() {
    let config = HippoxConfig::load_from_env();
    let mut global = GLOBAL_CONFIG.write().unwrap();
    *global = config;
}

/// init global configuration from TOML file
pub(crate) fn init_config_from_toml_file(path: &str) -> anyhow::Result<()> {
    let config = HippoxConfig::load_from_toml_file(path)?;
    let mut global = GLOBAL_CONFIG.write().unwrap();
    *global = config;
    Ok(())
}

/// init global configuration from JSON file
pub(crate) fn init_config_from_json_file(path: &str) -> anyhow::Result<()> {
    let config = HippoxConfig::load_from_json_file(path)?;
    let mut global = GLOBAL_CONFIG.write().unwrap();
    *global = config;
    Ok(())
}

/// init global configuration from optional parameters
#[allow(clippy::too_many_arguments)]
pub(crate) fn init_config_from_params(
    lang: Option<String>,
    provider: Option<String>,
    enable_cli: Option<bool>,
    enable_tcp: Option<bool>,
    enable_http: Option<bool>,
    enable_websocket: Option<bool>,
    enable_grpc: Option<bool>,
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    smtp_username: Option<String>,
    smtp_password: Option<String>,
    smtp_from: Option<String>,
    telegram_bot_token: Option<String>,
    dingding_access_token: Option<String>,
    feishu_webhook: Option<String>,
    wecom_webhook: Option<String>,
    // FTP parameters
    ftp_host: Option<String>,
    ftp_port: Option<u16>,
    ftp_username: Option<String>,
    ftp_password: Option<String>,
    ftp_remote_dir: Option<String>,
    ftp_timeout: Option<u64>,
    ftp_mode: Option<String>,
    // TCP parameters
    tcp_host: Option<String>,
    tcp_port: Option<u16>,
    tcp_timeout: Option<u64>,
    tcp_encoding: Option<String>,
    // UDP parameters
    udp_host: Option<String>,
    udp_port: Option<u16>,
    udp_timeout: Option<u64>,
    udp_encoding: Option<String>,
    udp_broadcast: Option<bool>,
    // PostgreSQL parameters
    pg_host: Option<String>,
    pg_port: Option<u16>,
    pg_database: Option<String>,
    pg_username: Option<String>,
    pg_password: Option<String>,
    pg_pool_size: Option<usize>,
    pg_timeout: Option<u64>,
    // MySQL parameters
    mysql_host: Option<String>,
    mysql_port: Option<u16>,
    mysql_database: Option<String>,
    mysql_username: Option<String>,
    mysql_password: Option<String>,
    mysql_pool_size: Option<usize>,
    mysql_timeout: Option<u64>,
    // Redis parameters
    redis_host: Option<String>,
    redis_port: Option<u16>,
    redis_password: Option<String>,
    redis_db: Option<usize>,
    redis_pool_size: Option<usize>,
    redis_timeout: Option<u64>,
    // SQLite parameters
    sqlite_path: Option<String>,
    sqlite_pool_size: Option<usize>,
    sqlite_timeout: Option<u64>,
    // GitHub parameters
    github_token: Option<String>,
    github_api_url: Option<String>,
    github_timeout: Option<u64>,
) {
    let config = HippoxConfig::load_from_params(
        lang,
        provider,
        enable_cli,
        enable_tcp,
        enable_http,
        enable_websocket,
        enable_grpc,
        smtp_host,
        smtp_port,
        smtp_username,
        smtp_password,
        smtp_from,
        telegram_bot_token,
        dingding_access_token,
        feishu_webhook,
        wecom_webhook,
        ftp_host,
        ftp_port,
        ftp_username,
        ftp_password,
        ftp_remote_dir,
        ftp_timeout,
        ftp_mode,
        tcp_host,
        tcp_port,
        tcp_timeout,
        tcp_encoding,
        udp_host,
        udp_port,
        udp_timeout,
        udp_encoding,
        udp_broadcast,
        // PostgreSQL parameters
        pg_host,
        pg_port,
        pg_database,
        pg_username,
        pg_password,
        pg_pool_size,
        pg_timeout,
        // MySQL parameters
        mysql_host,
        mysql_port,
        mysql_database,
        mysql_username,
        mysql_password,
        mysql_pool_size,
        mysql_timeout,
        // Redis parameters
        redis_host,
        redis_port,
        redis_password,
        redis_db,
        redis_pool_size,
        redis_timeout,
        // SQLite parameters
        sqlite_path,
        sqlite_pool_size,
        sqlite_timeout,
        github_token,
        github_api_url,
        github_timeout,
    );
    let mut global = GLOBAL_CONFIG.write().unwrap();
    *global = config;
}

/// init global configuration from JSON string of optional parameters
pub(crate) fn init_config_from_params_json_str(json_str: &str) -> anyhow::Result<()> {
    let config = HippoxConfig::load_from_params_json_str(json_str)?;
    let mut global = GLOBAL_CONFIG.write().unwrap();
    *global = config;
    Ok(())
}

/// Get a clone of the global configuration
pub fn get_config() -> HippoxConfig {
    GLOBAL_CONFIG.read().unwrap().clone()
}

// update config
pub fn update_config<F>(f: F) -> anyhow::Result<()>
where
    F: FnOnce(&mut HippoxConfig),
{
    let mut global = GLOBAL_CONFIG.write().unwrap();
    f(&mut global);
    Ok(())
}
