//! Sensitive file scan skill

use anyhow::Result;
use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashMap;

use crate::{
    SkillCategory,
    types::{Skill, SkillParameter},
};

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

#[derive(Debug)]
pub struct SensitiveFileScanSkill;

#[async_trait::async_trait]
impl Skill for SensitiveFileScanSkill {
    fn name(&self) -> &str {
        "sensitive_file_scan"
    }

    fn description(&self) -> &str {
        "Scan for sensitive files exposed on a web server"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill to find exposed sensitive files like .env, .git, config files, etc."
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        vec![
            SkillParameter {
                name: "target".to_string(),
                param_type: "string".to_string(),
                description: "Target URL (e.g., http://example.com)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("http://example.com".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(5.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "concurrency".to_string(),
                param_type: "integer".to_string(),
                description: "Number of concurrent requests".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(20.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "sensitive_file_scan",
            "parameters": {
                "target": "http://example.com"
            }
        })
    }

    fn example_output(&self) -> String {
        "Sensitive File Scan Results:\n\nFound: .env (200) - Environment configuration file\nFound: .git/config (200) - Git configuration file\nFound: robots.txt (200) - Robots exclusion file".to_string()
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Network
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let target = get_param_string(parameters, "target")?;
        let timeout_secs = get_param_u64(parameters, "timeout", 5);
        let concurrency = get_param_u64(parameters, "concurrency", 10) as usize;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()?;

        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));
        let mut tasks = vec![];

        for (path, desc) in SENSITIVE_FILES {
            let permit = semaphore.clone().acquire_owned().await?;
            let client_clone = client.clone();
            let target_clone = target.clone();
            let path_clone = path.to_string();
            let desc_clone = desc.to_string();

            tasks.push(tokio::spawn(async move {
                let url = format!("{}/{}", target_clone, path_clone);
                match client_clone.get(&url).send().await {
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        if status < 400 {
                            Some((url, status, desc_clone))
                        } else {
                            None
                        }
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

        let mut output = format!("Sensitive File Scan Results for {}:\n", target);
        if found.is_empty() {
            output.push_str("\nNo sensitive files found.");
        } else {
            output.push_str(&format!("\nFound {} sensitive files:\n", found.len()));
            for (url, status, desc) in found {
                output.push_str(&format!("  {} (HTTP {}) - {}\n", url, status, desc));
            }
        }

        Ok(output)
    }
}

fn get_param_string(params: &HashMap<String, Value>, name: &str) -> Result<String> {
    params
        .get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: {}", name))
}

fn get_param_u64(params: &HashMap<String, Value>, name: &str, default: u64) -> u64 {
    params.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
}
