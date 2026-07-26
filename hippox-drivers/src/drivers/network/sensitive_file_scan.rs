//! Sensitive file scan driver
//!
//! This driver provides functionality to scan for sensitive files exposed on a web server.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// List of sensitive files to scan for
const SENSITIVE_FILES: &[(&str, &str)] = &[
    (".env", "Environment configuration file"),
    (".git/config", "Git configuration file"),
    (".svn/entries", "SVN entries file"),
    (".htaccess", "Apache configuration file"),
    (".htpasswd", "Apache password file"),
    ("robots.txt", "Robots exclusion file"),
    ("sitemap.xml", "Sitemap file"),
    ("crossdomain.xml", "Cross-domain policy file"),
    ("phpinfo.php", "PHP info file"),
    ("php.ini", "PHP configuration file"),
    ("config.php", "PHP configuration file"),
    ("config.inc.php", "PHP configuration file"),
    ("wp-config.php", "WordPress configuration file"),
    (".gitignore", "Git ignore file"),
    ("composer.json", "Composer configuration file"),
    ("package.json", "Node.js package file"),
    ("requirements.txt", "Python requirements file"),
    ("Dockerfile", "Docker build file"),
    ("docker-compose.yml", "Docker compose file"),
    ("web.config", "IIS configuration file"),
    (".aws/credentials", "AWS credentials file"),
    (".ssh/id_rsa", "SSH private key"),
    (".ssh/id_dsa", "SSH private key"),
    (".ssh/id_ed25519", "SSH private key"),
    (".bash_history", "Bash history file"),
    (".mysql_history", "MySQL history file"),
    (".psql_history", "PostgreSQL history file"),
];
/// Driver for scanning sensitive files
#[derive(Debug)]
pub struct SensitiveFileScanDriver;
#[async_trait::async_trait]
impl Driver for SensitiveFileScanDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "sensitive_file_scan"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Scan for sensitive files exposed on a web server"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to find exposed sensitive files like .env, .git, config files, etc."
    }
    /// Returns the parameter definitions for this driver
    fn parameters(&self) -> Vec<DriverParameter> {
        return vec![
            DriverParameter {
                name: "target".to_string(),
                param_type: "string".to_string(),
                description: "Target URL (e.g., http://example.com)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("http://example.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(5.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "concurrency".to_string(),
                param_type: "integer".to_string(),
                description: "Number of concurrent requests".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(20.into())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "sensitive_file_scan",
            "parameters": {
                "target": "http://example.com"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Sensitive File Scan Results:\n\nFound: .env (200) - Environment configuration file\nFound: .git/config (200) - Git configuration file\nFound: robots.txt (200) - Robots exclusion file".to_string();
    }
    /// Returns the category of this driver
    fn category(&self) -> DriverCategory {
        return DriverCategory::Network;
    }
    /// Executes the driver with the given parameters
    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        _callback: Option<&dyn DriverCallback>,
        _context: Option<&DriverContext>,
    ) -> DriverResult<String> {
        debug!("Executing sensitive_file_scan driver");
        let target = get_param_string(parameters, "target")?;
        let timeout_secs = get_param_u64(parameters, "timeout", 5);
        let concurrency = get_param_u64(parameters, "concurrency", 10) as usize;
        info!("Sensitive file scan: target={}, timeout={}s, concurrency={}", target, timeout_secs, concurrency);
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| DriverError::execution(format!("Failed to build HTTP client: {}", e)))?;
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));
        let mut tasks = vec![];
        for (path, desc) in SENSITIVE_FILES {
            let permit =
                semaphore.clone().acquire_owned().await.map_err(|e| DriverError::execution(format!("Failed to acquire semaphore: {}", e)))?;
            let client_clone = client.clone();
            let target_clone = target.clone();
            let path_clone = path.to_string();
            let desc_clone = desc.to_string();
            tasks.push(tokio::spawn(async move {
                let url = format!("{}/{}", target_clone, path_clone);
                match client_clone.get(&url).send().await {
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        if status < 400 { Some((url, status, desc_clone)) } else { None }
                    }
                    _ => None,
                }
            }));
        }
        let mut found = Vec::new();
        for task in tasks {
            if let Ok(Some(result)) = task.await {
                found.push(result);
            }
        }
        info!("Sensitive file scan complete: {} files found", found.len());
        let mut output = format!("Sensitive File Scan Results for {}:\n", target);
        if found.is_empty() {
            output.push_str("\nNo sensitive files found.");
            info!("No sensitive files found");
        } else {
            output.push_str(&format!("\nFound {} sensitive files:\n", found.len()));
            for (url, status, desc) in found {
                output.push_str(&format!("  {} (HTTP {}) - {}\n", url, status, desc));
            }
        }
        return Ok(output);
    }
}
/// Gets a string parameter from the parameters map
fn get_param_string(params: &HashMap<String, Value>, name: &str) -> DriverResult<String> {
    params.get(name).and_then(|v| v.as_str()).map(|s| s.to_string()).ok_or_else(|| DriverError::missing_parameter(name))
}
/// Gets a u64 parameter from the parameters map with a default value
fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    params.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
}
