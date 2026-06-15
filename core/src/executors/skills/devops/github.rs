use crate::config::{get_github_instance, list_github_instances};
use crate::executors::types::{Skill, SkillParameter};
use crate::executors::{RequestConfig, execute};
use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;

/// Helper to get GitHub instance config
fn get_github_config(instance_id: Option<&str>) -> Result<crate::config::GitHubConfig> {
    if let Some(id) = instance_id {
        get_github_instance(id).ok_or_else(|| anyhow::anyhow!("GitHub instance not found: {}", id))
    } else {
        let instances = list_github_instances();
        instances.into_iter().next().ok_or_else(|| {
            anyhow::anyhow!("No GitHub instance configured. Please add a GitHub instance first.")
        })
    }
}

/// GitHub base helper
struct GitHubApi;

impl GitHubApi {
    fn build_url(endpoint: &str, config: &crate::config::GitHubConfig) -> String {
        format!("{}/{}", config.api_url.trim_end_matches('/'), endpoint)
    }

    fn build_headers(config: &crate::config::GitHubConfig) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert(
            "Accept".to_string(),
            "application/vnd.github.v3+json".to_string(),
        );
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", config.token),
        );
        headers.insert("User-Agent".to_string(), "Hippox-Engine".to_string());
        headers
    }

    async fn get(endpoint: &str, config: &crate::config::GitHubConfig) -> Result<String> {
        let req_config = RequestConfig {
            url: Self::build_url(endpoint, config),
            method: "GET".to_string(),
            headers: Some(Self::build_headers(config)),
            body: None,
            timeout_secs: Some(config.timeout),
        };
        let response = execute(&req_config).await?;
        if response.is_success {
            Ok(response.body)
        } else {
            anyhow::bail!("GitHub API error: {}", response.body)
        }
    }

    async fn post(
        endpoint: &str,
        body: &str,
        config: &crate::config::GitHubConfig,
    ) -> Result<String> {
        let req_config = RequestConfig {
            url: Self::build_url(endpoint, config),
            method: "POST".to_string(),
            headers: Some(Self::build_headers(config)),
            body: Some(body.to_string()),
            timeout_secs: Some(config.timeout),
        };
        let response = execute(&req_config).await?;
        if response.is_success {
            Ok(response.body)
        } else {
            anyhow::bail!("GitHub API error: {}", response.body)
        }
    }

    async fn put(
        endpoint: &str,
        body: Option<&str>,
        config: &crate::config::GitHubConfig,
    ) -> Result<String> {
        let req_config = RequestConfig {
            url: Self::build_url(endpoint, config),
            method: "PUT".to_string(),
            headers: Some(Self::build_headers(config)),
            body: body.map(|s| s.to_string()),
            timeout_secs: Some(config.timeout),
        };
        let response = execute(&req_config).await?;
        if response.is_success {
            Ok(response.body)
        } else {
            anyhow::bail!("GitHub API error: {}", response.body)
        }
    }

    async fn delete(endpoint: &str, config: &crate::config::GitHubConfig) -> Result<String> {
        let req_config = RequestConfig {
            url: Self::build_url(endpoint, config),
            method: "DELETE".to_string(),
            headers: Some(Self::build_headers(config)),
            body: None,
            timeout_secs: Some(config.timeout),
        };
        let response = execute(&req_config).await?;
        if response.is_success {
            Ok(response.body)
        } else {
            anyhow::bail!("GitHub API error: {}", response.body)
        }
    }
}

/// Get repository information
#[derive(Debug)]
pub struct GithubGetRepo;

#[async_trait::async_trait]
impl Skill for GithubGetRepo {
    fn name(&self) -> &str {
        "github_get_repo"
    }

    fn description(&self) -> &str {
        "Get information about a GitHub repository"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to get repository details like stars, forks, description"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_github_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "GitHub instance ID (use 'list_github_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("github_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner (username or organization)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub token (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "github_get_repo",
            "parameters": {
                "instance_id": "github_prod",
                "owner": "rust-lang",
                "repo": "rust"
            }
        })
    }

    fn example_output(&self) -> String {
        r#"{"name": "rust", "full_name": "rust-lang/rust", "description": "Empowering everyone...", "stargazers_count": 85000, "forks_count": 11000} [instance: github_prod]"#.to_string()
    }

    fn category(&self) -> &str {
        "github"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_github_config(instance_id)?;

        if let Some(token) = parameters.get("token").and_then(|v| v.as_str()) {
            instance.token = token.to_string();
        }
        if let Some(api_url) = parameters.get("api_url").and_then(|v| v.as_str()) {
            instance.api_url = api_url.to_string();
        }

        let owner = parameters
            .get("owner")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: owner"))?;
        let repo = parameters
            .get("repo")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: repo"))?;

        let endpoint = format!("repos/{}/{}", owner, repo);
        let result = GitHubApi::get(&endpoint, &instance).await?;

        Ok(format!("{} [instance: {}]", result, instance.name))
    }
}

/// Create an issue
#[derive(Debug)]
pub struct GithubCreateIssue;

#[async_trait::async_trait]
impl Skill for GithubCreateIssue {
    fn name(&self) -> &str {
        "github_create_issue"
    }

    fn description(&self) -> &str {
        "Create an issue in a GitHub repository"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to report a bug or request a feature"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_github_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "GitHub instance ID (use 'list_github_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("github_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Issue title".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Bug: compilation error".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "body".to_string(),
                param_type: "string".to_string(),
                description: "Issue body/description".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("When compiling with nightly...".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "labels".to_string(),
                param_type: "array".to_string(),
                description: "Labels to apply".to_string(),
                required: false,
                default: Some(Value::Array(vec![])),
                example: Some(json!(["bug", "help-wanted"])),
                enum_values: None,
            },
            SkillParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub token (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "github_create_issue",
            "parameters": {
                "instance_id": "github_prod",
                "owner": "rust-lang",
                "repo": "rust",
                "title": "Bug: compilation error",
                "body": "When compiling with nightly...",
                "labels": ["bug"]
            }
        })
    }

    fn example_output(&self) -> String {
        r#"{"number": 12345, "html_url": "https://github.com/rust-lang/rust/issues/12345"} [instance: github_prod]"#
            .to_string()
    }

    fn category(&self) -> &str {
        "github"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_github_config(instance_id)?;

        if let Some(token) = parameters.get("token").and_then(|v| v.as_str()) {
            instance.token = token.to_string();
        }
        if let Some(api_url) = parameters.get("api_url").and_then(|v| v.as_str()) {
            instance.api_url = api_url.to_string();
        }

        let owner = parameters
            .get("owner")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: owner"))?;
        let repo = parameters
            .get("repo")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: repo"))?;
        let title = parameters
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: title"))?;

        let mut body_json = json!({
            "title": title,
        });
        if let Some(b) = parameters.get("body").and_then(|v| v.as_str()) {
            body_json["body"] = json!(b);
        }
        if let Some(labels) = parameters.get("labels").and_then(|v| v.as_array()) {
            let label_strings: Vec<String> = labels
                .iter()
                .filter_map(|l| l.as_str())
                .map(|s| s.to_string())
                .collect();
            body_json["labels"] = json!(label_strings);
        }

        let endpoint = format!("repos/{}/{}/issues", owner, repo);
        let result = GitHubApi::post(&endpoint, &body_json.to_string(), &instance).await?;

        Ok(format!("{} [instance: {}]", result, instance.name))
    }
}

/// List issues
#[derive(Debug)]
pub struct GithubListIssues;

#[async_trait::async_trait]
impl Skill for GithubListIssues {
    fn name(&self) -> &str {
        "github_list_issues"
    }

    fn description(&self) -> &str {
        "List issues from a GitHub repository"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to see existing issues"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_github_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "GitHub instance ID (use 'list_github_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("github_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "state".to_string(),
                param_type: "string".to_string(),
                description: "Issue state (open, closed, all)".to_string(),
                required: false,
                default: Some(Value::String("open".to_string())),
                example: Some(Value::String("open".to_string())),
                enum_values: Some(vec![
                    "open".to_string(),
                    "closed".to_string(),
                    "all".to_string(),
                ]),
            },
            SkillParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of issues to return".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub token (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "github_list_issues",
            "parameters": {
                "instance_id": "github_prod",
                "owner": "rust-lang",
                "repo": "rust",
                "state": "open",
                "limit": 10
            }
        })
    }

    fn example_output(&self) -> String {
        r#"[{"number": 12345, "title": "Bug report", "state": "open"}] [instance: github_prod]"#
            .to_string()
    }

    fn category(&self) -> &str {
        "github"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_github_config(instance_id)?;

        if let Some(token) = parameters.get("token").and_then(|v| v.as_str()) {
            instance.token = token.to_string();
        }
        if let Some(api_url) = parameters.get("api_url").and_then(|v| v.as_str()) {
            instance.api_url = api_url.to_string();
        }

        let owner = parameters
            .get("owner")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: owner"))?;
        let repo = parameters
            .get("repo")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: repo"))?;
        let state = parameters
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("open");
        let limit = parameters
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        let endpoint = format!(
            "repos/{}/{}/issues?state={}&per_page={}",
            owner, repo, state, limit
        );
        let result = GitHubApi::get(&endpoint, &instance).await?;

        Ok(format!("{} [instance: {}]", result, instance.name))
    }
}

/// Star a repository
#[derive(Debug)]
pub struct GithubStarRepo;

#[async_trait::async_trait]
impl Skill for GithubStarRepo {
    fn name(&self) -> &str {
        "github_star_repo"
    }

    fn description(&self) -> &str {
        "Star a GitHub repository"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to star/favorite a repository"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_github_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "GitHub instance ID (use 'list_github_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("github_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub token (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "github_star_repo",
            "parameters": {
                "instance_id": "github_prod",
                "owner": "rust-lang",
                "repo": "rust"
            }
        })
    }

    fn example_output(&self) -> String {
        "Successfully starred rust-lang/rust [instance: github_prod]".to_string()
    }

    fn category(&self) -> &str {
        "github"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_github_config(instance_id)?;

        if let Some(token) = parameters.get("token").and_then(|v| v.as_str()) {
            instance.token = token.to_string();
        }
        if let Some(api_url) = parameters.get("api_url").and_then(|v| v.as_str()) {
            instance.api_url = api_url.to_string();
        }

        let owner = parameters
            .get("owner")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: owner"))?;
        let repo = parameters
            .get("repo")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: repo"))?;

        let endpoint = format!("user/starred/{}/{}", owner, repo);
        GitHubApi::put(&endpoint, None, &instance).await?;

        Ok(format!(
            "Successfully starred {}/{} [instance: {}]",
            owner, repo, instance.name
        ))
    }
}

/// Search repositories
#[derive(Debug)]
pub struct GithubSearchRepos;

#[async_trait::async_trait]
impl Skill for GithubSearchRepos {
    fn name(&self) -> &str {
        "github_search_repos"
    }

    fn description(&self) -> &str {
        "Search GitHub repositories by query"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to find repositories"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_github_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "GitHub instance ID (use 'list_github_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("github_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "Search query (e.g., 'rust language:rust')".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust language:rust".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of results".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub token (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "github_search_repos",
            "parameters": {
                "instance_id": "github_prod",
                "query": "rust language:rust",
                "limit": 5
            }
        })
    }

    fn example_output(&self) -> String {
        r#"{"total_count": 12345, "items": [{"full_name": "rust-lang/rust", "description": "..."}]} [instance: github_prod]"#.to_string()
    }

    fn category(&self) -> &str {
        "github"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_github_config(instance_id)?;

        if let Some(token) = parameters.get("token").and_then(|v| v.as_str()) {
            instance.token = token.to_string();
        }
        if let Some(api_url) = parameters.get("api_url").and_then(|v| v.as_str()) {
            instance.api_url = api_url.to_string();
        }

        let query = parameters
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;
        let limit = parameters
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);

        let encoded_query = urlencoding::encode(query);
        let endpoint = format!("search/repositories?q={}&per_page={}", encoded_query, limit);
        let result = GitHubApi::get(&endpoint, &instance).await?;

        Ok(format!("{} [instance: {}]", result, instance.name))
    }
}

/// Get user information
#[derive(Debug)]
pub struct GithubGetUser;

#[async_trait::async_trait]
impl Skill for GithubGetUser {
    fn name(&self) -> &str {
        "github_get_user"
    }

    fn description(&self) -> &str {
        "Get GitHub user information"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to get profile info of a GitHub user"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_github_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "GitHub instance ID (use 'list_github_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("github_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "GitHub username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("octocat".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub token (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "github_get_user",
            "parameters": {
                "instance_id": "github_prod",
                "username": "octocat"
            }
        })
    }

    fn example_output(&self) -> String {
        r#"{"login": "octocat", "name": "The Octocat", "public_repos": 8} [instance: github_prod]"#
            .to_string()
    }

    fn category(&self) -> &str {
        "github"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_github_config(instance_id)?;

        if let Some(token) = parameters.get("token").and_then(|v| v.as_str()) {
            instance.token = token.to_string();
        }
        if let Some(api_url) = parameters.get("api_url").and_then(|v| v.as_str()) {
            instance.api_url = api_url.to_string();
        }

        let username = parameters
            .get("username")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: username"))?;

        let endpoint = format!("users/{}", username);
        let result = GitHubApi::get(&endpoint, &instance).await?;

        Ok(format!("{} [instance: {}]", result, instance.name))
    }
}

/// Get pull requests
#[derive(Debug)]
pub struct GithubListPRs;

#[async_trait::async_trait]
impl Skill for GithubListPRs {
    fn name(&self) -> &str {
        "github_list_prs"
    }

    fn description(&self) -> &str {
        "List pull requests from a GitHub repository"
    }

    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to see open pull requests"
    }

    fn parameters(&self) -> Vec<SkillParameter> {
        let instances = list_github_instances();
        let instance_ids: Vec<String> = instances.iter().map(|c| c.id.clone()).collect();

        vec![
            SkillParameter {
                name: "instance_id".to_string(),
                param_type: "string".to_string(),
                description:
                    "GitHub instance ID (use 'list_github_instances' to see available instances)"
                        .to_string(),
                required: false,
                default: if instance_ids.is_empty() {
                    None
                } else {
                    Some(Value::String(instance_ids[0].clone()))
                },
                example: Some(Value::String("github_prod".to_string())),
                enum_values: if instance_ids.is_empty() {
                    None
                } else {
                    Some(instance_ids)
                },
            },
            SkillParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "state".to_string(),
                param_type: "string".to_string(),
                description: "PR state (open, closed, all)".to_string(),
                required: false,
                default: Some(Value::String("open".to_string())),
                example: Some(Value::String("open".to_string())),
                enum_values: Some(vec![
                    "open".to_string(),
                    "closed".to_string(),
                    "all".to_string(),
                ]),
            },
            SkillParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of PRs to return".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            SkillParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub token (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            SkillParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL (overrides instance config)".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({
            "action": "github_list_prs",
            "parameters": {
                "instance_id": "github_prod",
                "owner": "rust-lang",
                "repo": "rust",
                "state": "open",
                "limit": 10
            }
        })
    }

    fn example_output(&self) -> String {
        r#"[{"number": 123, "title": "Add feature", "user": {"login": "contributor"}}] [instance: github_prod]"#.to_string()
    }

    fn category(&self) -> &str {
        "github"
    }

    async fn execute(&self, parameters: &HashMap<String, Value>) -> Result<String> {
        let instance_id = parameters.get("instance_id").and_then(|v| v.as_str());
        let mut instance = get_github_config(instance_id)?;

        if let Some(token) = parameters.get("token").and_then(|v| v.as_str()) {
            instance.token = token.to_string();
        }
        if let Some(api_url) = parameters.get("api_url").and_then(|v| v.as_str()) {
            instance.api_url = api_url.to_string();
        }

        let owner = parameters
            .get("owner")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: owner"))?;
        let repo = parameters
            .get("repo")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: repo"))?;
        let state = parameters
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("open");
        let limit = parameters
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        let endpoint = format!(
            "repos/{}/{}/pulls?state={}&per_page={}",
            owner, repo, state, limit
        );
        let result = GitHubApi::get(&endpoint, &instance).await?;

        Ok(format!("{} [instance: {}]", result, instance.name))
    }
}
