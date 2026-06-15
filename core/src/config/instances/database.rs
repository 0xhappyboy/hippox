//! Database instance configurations

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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
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
