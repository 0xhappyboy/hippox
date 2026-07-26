//! Directory scanning driver
//!
//! This driver provides functionality to scan for common directories and files on a web server.
use crate::{
    DriverCallback, DriverCategory, DriverContext, DriverError, DriverResult,
    types::{Driver, DriverParameter},
};
use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashMap;
use tracing::{debug, info};
/// Common directory names to scan
const COMMON_DIRS: &[&str] = &[
    "admin",
    "api",
    "assets",
    "backup",
    "blog",
    "css",
    "data",
    "docs",
    "download",
    "files",
    "images",
    "img",
    "includes",
    "js",
    "lib",
    "media",
    "private",
    "public",
    "scripts",
    "static",
    "styles",
    "temp",
    "tmp",
    "upload",
    "uploads",
    "vendor",
    ".git",
    ".svn",
    ".env",
    ".htaccess",
    ".htpasswd",
];
/// Common file names to scan
const COMMON_FILES: &[&str] = &[
    "index",
    "index.html",
    "index.php",
    "default",
    "default.html",
    "default.php",
    "main",
    "main.html",
    "main.php",
    "home",
    "home.html",
    "home.php",
    "robots.txt",
    "sitemap.xml",
    "sitemap.txt",
    "favicon.ico",
    "crossdomain.xml",
    "phpinfo.php",
    "phpinfo",
    "test.php",
    "test.html",
    "config.php",
    "config.ini",
    ".gitignore",
    "README.md",
    "README",
    "CHANGELOG.md",
    "CHANGELOG",
    "composer.json",
    "package.json",
    "requirements.txt",
    "Dockerfile",
];
/// Driver for scanning directories on a web server
#[derive(Debug)]
pub struct DirScanDriver;
#[async_trait::async_trait]
impl Driver for DirScanDriver {
    /// Returns the unique name of this driver
    fn name(&self) -> &str {
        "dir_scan"
    }
    /// Returns a brief description of the driver's functionality
    fn description(&self) -> &str {
        "Scan for common directories and files on a web server"
    }
    /// Returns detailed usage guidance for LLMs
    fn usage_hint(&self) -> &str {
        "Use this skill to discover hidden directories and files on a web server"
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
                name: "wordlist".to_string(),
                param_type: "string".to_string(),
                description: "Custom wordlist (comma-separated)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("admin,api,test".to_string())),
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
            DriverParameter {
                name: "file_ext".to_string(),
                param_type: "string".to_string(),
                description: "File extensions to check (comma-separated)".to_string(),
                required: false,
                default: Some(Value::String("html,php,txt".to_string())),
                example: Some(Value::String("html,php,json".to_string())),
                enum_values: None,
            },
        ];
    }
    /// Returns an example call for this driver
    fn example_call(&self) -> DriverResult<Value> {
        return Ok(json!({
            "action": "dir_scan",
            "parameters": {
                "target": "http://example.com"
            }
        }));
    }
    /// Returns an example output from this driver
    fn example_output(&self) -> String {
        return "Directory Scan Results:\n\nFound: http://example.com/admin/ (200)\nFound: http://example.com/.git/ (403)\nFound: http://example.com/robots.txt (200)".to_string();
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
        debug!("Executing dir_scan driver");
        let target = get_param_string(parameters, "target")?;
        let timeout_secs = get_param_u64(parameters, "timeout", 5);
        let concurrency = get_param_u64(parameters, "concurrency", 10) as usize;
        let file_exts: Vec<String> =
            parameters.get("file_ext").and_then(|v| v.as_str()).unwrap_or("html,php,txt").split(',').map(|s| s.trim().to_string()).collect();
        let mut wordlist: Vec<String> =
            parameters.get("wordlist").and_then(|v| v.as_str()).map(|s| s.split(',').map(|w| w.trim().to_string()).collect()).unwrap_or_else(|| {
                let mut list: Vec<String> = COMMON_DIRS.iter().map(|s| s.to_string()).collect();
                list.extend(COMMON_FILES.iter().map(|s| s.to_string()));
                list
            });
        wordlist.sort();
        wordlist.dedup();
        info!("Scanning target: {}, wordlist size: {}, concurrency: {}", target, wordlist.len(), concurrency);
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| DriverError::execution(format!("Failed to build HTTP client: {}", e)))?;
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));
        let mut tasks = vec![];
        let target_clone = target.clone();
        for path in wordlist {
            let permit =
                semaphore.clone().acquire_owned().await.map_err(|e| DriverError::execution(format!("Failed to acquire semaphore: {}", e)))?;
            let client_clone = client.clone();
            let target_clone = target_clone.clone();
            let ext_clone = file_exts.to_vec();
            tasks.push(tokio::spawn(async move {
                let mut results = Vec::new();
                let base_url = format!("{}/{}", target_clone, path);
                for ext in &ext_clone {
                    let url = format!("{}.{}", base_url, ext);
                    match client_clone.get(&url).send().await {
                        Ok(resp) => {
                            let status = resp.status().as_u16();
                            if status < 400 {
                                results.push((url.clone(), status));
                            }
                        }
                        _ => {}
                    }
                }
                match client_clone.get(&base_url).send().await {
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        if status < 400 {
                            results.push((base_url.clone(), status));
                        }
                    }
                    _ => {}
                }
                drop(permit);
                results
            }));
        }
        let mut found = Vec::new();
        for task in tasks {
            if let Ok(results) = task.await {
                found.extend(results);
            }
        }
        found.sort_by(|a, b| a.0.cmp(&b.0));
        found.dedup();
        info!("Found {} items during directory scan", found.len());
        let mut output = format!("Directory Scan Results for {}:\n", target);
        if found.is_empty() {
            output.push_str("\nNo directories or files found.");
            info!("No directories or files found");
        } else {
            output.push_str(&format!("\nFound {} items:\n", found.len()));
            for (url, status) in found {
                output.push_str(&format!("  {} (HTTP {})\n", url, status));
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
