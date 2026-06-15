//! Container orchestration configurations

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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
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
        let id_clone = id.clone();
        Self {
            id,
            name: name.unwrap_or_else(|| id_clone),
            description: description.unwrap_or_default(),
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
