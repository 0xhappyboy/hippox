//! Notification service configurations

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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
            bot_token,
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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
            access_token,
            secret: None,
        }
    }

    pub fn with_secret(mut self, secret: String) -> Self {
        self.secret = Some(secret);
        self
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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
            webhook,
            secret: None,
        }
    }

    pub fn with_secret(mut self, secret: String) -> Self {
        self.secret = Some(secret);
        self
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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
            webhook,
            key: None,
        }
    }

    pub fn with_key(mut self, key: String) -> Self {
        self.key = Some(key);
        self
    }
}
