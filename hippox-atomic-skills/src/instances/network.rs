//! Network instance configurations

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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
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
