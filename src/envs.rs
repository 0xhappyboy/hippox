/// Language setting (en/zh)
pub const HIPPOX_LANG: &str = "HIPPOX_LANG";

/// SMTP server hostname (e.g., smtp.gmail.com)
pub const HIPPOX_SMTP_HOST: &str = "HIPPOX_SMTP_HOST";

/// SMTP server port (e.g., 587 for TLS, 465 for SSL)
pub const HIPPOX_SMTP_PORT: &str = "HIPPOX_SMTP_PORT";

/// SMTP authentication username (usually email address)
pub const HIPPOX_SMTP_USERNAME: &str = "HIPPOX_SMTP_USERNAME";

/// SMTP authentication password or app-specific password
pub const HIPPOX_SMTP_PASSWORD: &str = "HIPPOX_SMTP_PASSWORD";

/// Default sender email address
pub const HIPPOX_SMTP_FROM: &str = "HIPPOX_SMTP_FROM";

/// Telegram bot token (format: 1234567890:ABCdefGHIJKLMNopqrsTUVwxyz)
pub const HIPPOX_TELEGRAM_BOT_TOKEN: &str = "HIPPOX_TELEGRAM_BOT_TOKEN";

/// DingDing robot access token
pub const HIPPOX_DINGDING_ACCESS_TOKEN: &str = "HIPPOX_DINGDING_ACCESS_TOKEN";

/// Feishu bot webhook URL
pub const HIPPOX_FEISHU_WEBHOOK: &str = "HIPPOX_FEISHU_WEBHOOK";

/// WeCom (Enterprise WeChat) robot webhook URL
pub const HIPPOX_WECOM_WEBHOOK: &str = "HIPPOX_WECOM_WEBHOOK";

// ==================== FTP Configuration ====================
/// FTP server hostname
pub const HIPPOX_FTP_HOST: &str = "HIPPOX_FTP_HOST";
/// FTP server port
pub const HIPPOX_FTP_PORT: &str = "HIPPOX_FTP_PORT";
/// FTP username
pub const HIPPOX_FTP_USERNAME: &str = "HIPPOX_FTP_USERNAME";
/// FTP password
pub const HIPPOX_FTP_PASSWORD: &str = "HIPPOX_FTP_PASSWORD";
/// FTP default remote directory
pub const HIPPOX_FTP_REMOTE_DIR: &str = "HIPPOX_FTP_REMOTE_DIR";
/// FTP connection timeout (seconds)
pub const HIPPOX_FTP_TIMEOUT: &str = "HIPPOX_FTP_TIMEOUT";
/// FTP transfer mode (binary/ascii)
pub const HIPPOX_FTP_MODE: &str = "HIPPOX_FTP_MODE";

// ==================== TCP Configuration ====================
/// TCP default host
pub const HIPPOX_TCP_HOST: &str = "HIPPOX_TCP_HOST";
/// TCP default port
pub const HIPPOX_TCP_PORT: &str = "HIPPOX_TCP_PORT";
/// TCP default timeout (seconds)
pub const HIPPOX_TCP_TIMEOUT: &str = "HIPPOX_TCP_TIMEOUT";
/// TCP default encoding (utf8, hex, base64)
pub const HIPPOX_TCP_ENCODING: &str = "HIPPOX_TCP_ENCODING";

// ==================== UDP Configuration ====================
/// UDP default host
pub const HIPPOX_UDP_HOST: &str = "HIPPOX_UDP_HOST";
/// UDP default port
pub const HIPPOX_UDP_PORT: &str = "HIPPOX_UDP_PORT";
/// UDP default timeout (seconds)
pub const HIPPOX_UDP_TIMEOUT: &str = "HIPPOX_UDP_TIMEOUT";
/// UDP default encoding (utf8, hex, base64)
pub const HIPPOX_UDP_ENCODING: &str = "HIPPOX_UDP_ENCODING";
/// UDP enable broadcast
pub const HIPPOX_UDP_BROADCAST: &str = "HIPPOX_UDP_BROADCAST";

// ==================== PostgreSQL Configuration ====================
pub const HIPPOX_PG_HOST: &str = "HIPPOX_PG_HOST";
pub const HIPPOX_PG_PORT: &str = "HIPPOX_PG_PORT";
pub const HIPPOX_PG_DATABASE: &str = "HIPPOX_PG_DATABASE";
pub const HIPPOX_PG_USERNAME: &str = "HIPPOX_PG_USERNAME";
pub const HIPPOX_PG_PASSWORD: &str = "HIPPOX_PG_PASSWORD";
pub const HIPPOX_PG_POOL_SIZE: &str = "HIPPOX_PG_POOL_SIZE";
pub const HIPPOX_PG_TIMEOUT: &str = "HIPPOX_PG_TIMEOUT";

// ==================== MySQL Configuration ====================
pub const HIPPOX_MYSQL_HOST: &str = "HIPPOX_MYSQL_HOST";
pub const HIPPOX_MYSQL_PORT: &str = "HIPPOX_MYSQL_PORT";
pub const HIPPOX_MYSQL_DATABASE: &str = "HIPPOX_MYSQL_DATABASE";
pub const HIPPOX_MYSQL_USERNAME: &str = "HIPPOX_MYSQL_USERNAME";
pub const HIPPOX_MYSQL_PASSWORD: &str = "HIPPOX_MYSQL_PASSWORD";
pub const HIPPOX_MYSQL_POOL_SIZE: &str = "HIPPOX_MYSQL_POOL_SIZE";
pub const HIPPOX_MYSQL_TIMEOUT: &str = "HIPPOX_MYSQL_TIMEOUT";

// ==================== Redis Configuration ====================
pub const HIPPOX_REDIS_HOST: &str = "HIPPOX_REDIS_HOST";
pub const HIPPOX_REDIS_PORT: &str = "HIPPOX_REDIS_PORT";
pub const HIPPOX_REDIS_PASSWORD: &str = "HIPPOX_REDIS_PASSWORD";
pub const HIPPOX_REDIS_DB: &str = "HIPPOX_REDIS_DB";
pub const HIPPOX_REDIS_POOL_SIZE: &str = "HIPPOX_REDIS_POOL_SIZE";
pub const HIPPOX_REDIS_TIMEOUT: &str = "HIPPOX_REDIS_TIMEOUT";

// ==================== SQLite Configuration ====================
pub const HIPPOX_SQLITE_PATH: &str = "HIPPOX_SQLITE_PATH";
pub const HIPPOX_SQLITE_POOL_SIZE: &str = "HIPPOX_SQLITE_POOL_SIZE";
pub const HIPPOX_SQLITE_TIMEOUT: &str = "HIPPOX_SQLITE_TIMEOUT";

// ==================== GitHub Configuration ====================
/// GitHub Personal Access Token
pub const HIPPOX_GITHUB_TOKEN: &str = "HIPPOX_GITHUB_TOKEN";
/// GitHub API URL (default: https://api.github.com)
pub const HIPPOX_GITHUB_API_URL: &str = "HIPPOX_GITHUB_API_URL";
/// GitHub API timeout in seconds
pub const HIPPOX_GITHUB_TIMEOUT: &str = "HIPPOX_GITHUB_TIMEOUT";

/// Get environment variable value with optional default
pub fn get_env(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// Get environment variable or return default
pub fn get_env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Check if environment variable is set to "true"
pub fn is_env_true(key: &str) -> bool {
    std::env::var(key).unwrap_or_else(|_| "false".to_string()) == "true"
}

/// Get required environment variable, returns error if not set
pub fn get_required_env(key: &str) -> anyhow::Result<String> {
    std::env::var(key).map_err(|_| anyhow::anyhow!("Environment variable '{}' is not set", key))
}
