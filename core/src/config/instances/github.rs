//! GitHub configuration

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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
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
