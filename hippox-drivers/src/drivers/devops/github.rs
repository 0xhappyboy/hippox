// github.rs
//! GitHub API skills

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::DriverCallback;
use crate::DriverContext;
use crate::types::{Driver, DriverParameter};
use crate::{RequestConfig, DriverCategory, execute};

// ========== Helper functions ==========

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

fn get_param_array(params: &HashMap<String, Value>, name: &str) -> Vec<Value> {
    params
        .get(name)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

fn get_param_bool(params: &HashMap<String, Value>, name: &str, default: bool) -> bool {
    params
        .get(name)
        .and_then(|v| v.as_bool())
        .unwrap_or(default)
}

// ========== GitHub API Helper ==========

struct GitHubApi;

impl GitHubApi {
    fn build_url(api_url: &str, endpoint: &str) -> String {
        format!("{}/{}", api_url.trim_end_matches('/'), endpoint)
    }

    fn build_headers(token: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert(
            "Accept".to_string(),
            "application/vnd.github.v3+json".to_string(),
        );
        headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        headers.insert("User-Agent".to_string(), "Hippox-Engine".to_string());
        headers
    }

    async fn get(endpoint: &str, token: &str, api_url: &str, timeout: u64) -> Result<String> {
        let req_config = RequestConfig {
            url: Self::build_url(api_url, endpoint),
            method: "GET".to_string(),
            headers: Some(Self::build_headers(token)),
            body: None,
            timeout_secs: Some(timeout),
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
        token: &str,
        api_url: &str,
        timeout: u64,
    ) -> Result<String> {
        let req_config = RequestConfig {
            url: Self::build_url(api_url, endpoint),
            method: "POST".to_string(),
            headers: Some(Self::build_headers(token)),
            body: Some(body.to_string()),
            timeout_secs: Some(timeout),
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
        token: &str,
        api_url: &str,
        timeout: u64,
    ) -> Result<String> {
        let req_config = RequestConfig {
            url: Self::build_url(api_url, endpoint),
            method: "PUT".to_string(),
            headers: Some(Self::build_headers(token)),
            body: body.map(|s| s.to_string()),
            timeout_secs: Some(timeout),
        };
        let response = execute(&req_config).await?;
        if response.is_success {
            Ok(response.body)
        } else {
            anyhow::bail!("GitHub API error: {}", response.body)
        }
    }

    async fn delete(endpoint: &str, token: &str, api_url: &str, timeout: u64) -> Result<String> {
        let req_config = RequestConfig {
            url: Self::build_url(api_url, endpoint),
            method: "DELETE".to_string(),
            headers: Some(Self::build_headers(token)),
            body: None,
            timeout_secs: Some(timeout),
        };
        let response = execute(&req_config).await?;
        if response.is_success {
            Ok(response.body)
        } else {
            anyhow::bail!("GitHub API error: {}", response.body)
        }
    }
}

// ========== Get repository information ==========
#[derive(Debug)]
pub struct GithubGetRepo;

#[async_trait::async_trait]
impl Driver for GithubGetRepo {
    fn name(&self) -> &str {
        "github_get_repo"
    }
    fn description(&self) -> &str {
        "Get information about a GitHub repository"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to get repository details like stars, forks, description"
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::Devops
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub personal access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner (username or organization)".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL (default: https://api.github.com)".to_string(),
                required: false,
                default: Some(Value::String("https://api.github.com".to_string())),
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "github_get_repo", "parameters": { "token": "ghp_xxxxxxxx", "owner": "rust-lang", "repo": "rust" } })
    }

    fn example_output(&self) -> String {
        r#"{"name": "rust", "full_name": "rust-lang/rust", "description": "Empowering everyone...", "stargazers_count": 85000, "forks_count": 11000}"#.to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let token = get_param_string(parameters, "token")?;
        let owner = get_param_string(parameters, "owner")?;
        let repo = get_param_string(parameters, "repo")?;
        let api_url = parameters
            .get("api_url")
            .and_then(|v| v.as_str())
            .unwrap_or("https://api.github.com");
        let timeout = get_param_u64(parameters, "timeout", 30);

        let endpoint = format!("repos/{}/{}", owner, repo);
        GitHubApi::get(&endpoint, &token, api_url, timeout).await
    }
}

// ========== Create an issue ==========
#[derive(Debug)]
pub struct GithubCreateIssue;

#[async_trait::async_trait]
impl Driver for GithubCreateIssue {
    fn name(&self) -> &str {
        "github_create_issue"
    }
    fn description(&self) -> &str {
        "Create an issue in a GitHub repository"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to report a bug or request a feature"
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::Devops
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub personal access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "title".to_string(),
                param_type: "string".to_string(),
                description: "Issue title".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("Bug: compilation error".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "body".to_string(),
                param_type: "string".to_string(),
                description: "Issue body/description".to_string(),
                required: false,
                default: None,
                example: Some(Value::String("When compiling with nightly...".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "labels".to_string(),
                param_type: "array".to_string(),
                description: "Labels to apply".to_string(),
                required: false,
                default: Some(Value::Array(vec![])),
                example: Some(json!(["bug", "help-wanted"])),
                enum_values: None,
            },
            DriverParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL".to_string(),
                required: false,
                default: Some(Value::String("https://api.github.com".to_string())),
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "github_create_issue", "parameters": { "token": "ghp_xxxxxxxx", "owner": "rust-lang", "repo": "rust", "title": "Bug: compilation error", "body": "When compiling with nightly...", "labels": ["bug"] } })
    }

    fn example_output(&self) -> String {
        r#"{"number": 12345, "html_url": "https://github.com/rust-lang/rust/issues/12345"}"#
            .to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let token = get_param_string(parameters, "token")?;
        let owner = get_param_string(parameters, "owner")?;
        let repo = get_param_string(parameters, "repo")?;
        let title = get_param_string(parameters, "title")?;
        let body = parameters.get("body").and_then(|v| v.as_str());
        let labels = get_param_array(parameters, "labels");
        let api_url = parameters
            .get("api_url")
            .and_then(|v| v.as_str())
            .unwrap_or("https://api.github.com");
        let timeout = get_param_u64(parameters, "timeout", 30);

        let mut body_json = json!({ "title": title });
        if let Some(b) = body {
            body_json["body"] = json!(b);
        }
        if !labels.is_empty() {
            let label_strings: Vec<String> = labels
                .iter()
                .filter_map(|l| l.as_str())
                .map(|s| s.to_string())
                .collect();
            body_json["labels"] = json!(label_strings);
        }

        let endpoint = format!("repos/{}/{}/issues", owner, repo);
        GitHubApi::post(&endpoint, &body_json.to_string(), &token, api_url, timeout).await
    }
}

// ========== List issues ==========
#[derive(Debug)]
pub struct GithubListIssues;

#[async_trait::async_trait]
impl Driver for GithubListIssues {
    fn name(&self) -> &str {
        "github_list_issues"
    }
    fn description(&self) -> &str {
        "List issues from a GitHub repository"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to see existing issues"
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::Devops
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub personal access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            DriverParameter {
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
            DriverParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of issues to return".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL".to_string(),
                required: false,
                default: Some(Value::String("https://api.github.com".to_string())),
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "github_list_issues", "parameters": { "token": "ghp_xxxxxxxx", "owner": "rust-lang", "repo": "rust", "state": "open", "limit": 10 } })
    }

    fn example_output(&self) -> String {
        r#"[{"number": 12345, "title": "Bug report", "state": "open"}]"#.to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let token = get_param_string(parameters, "token")?;
        let owner = get_param_string(parameters, "owner")?;
        let repo = get_param_string(parameters, "repo")?;
        let state = parameters
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("open");
        let limit = get_param_u64(parameters, "limit", 30);
        let api_url = parameters
            .get("api_url")
            .and_then(|v| v.as_str())
            .unwrap_or("https://api.github.com");
        let timeout = get_param_u64(parameters, "timeout", 30);

        let endpoint = format!(
            "repos/{}/{}/issues?state={}&per_page={}",
            owner, repo, state, limit
        );
        GitHubApi::get(&endpoint, &token, api_url, timeout).await
    }
}

// ========== Star a repository ==========
#[derive(Debug)]
pub struct GithubStarRepo;

#[async_trait::async_trait]
impl Driver for GithubStarRepo {
    fn name(&self) -> &str {
        "github_star_repo"
    }
    fn description(&self) -> &str {
        "Star a GitHub repository"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user wants to star/favorite a repository"
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::Devops
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub personal access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL".to_string(),
                required: false,
                default: Some(Value::String("https://api.github.com".to_string())),
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "github_star_repo", "parameters": { "token": "ghp_xxxxxxxx", "owner": "rust-lang", "repo": "rust" } })
    }

    fn example_output(&self) -> String {
        "Successfully starred rust-lang/rust".to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let token = get_param_string(parameters, "token")?;
        let owner = get_param_string(parameters, "owner")?;
        let repo = get_param_string(parameters, "repo")?;
        let api_url = parameters
            .get("api_url")
            .and_then(|v| v.as_str())
            .unwrap_or("https://api.github.com");
        let timeout = get_param_u64(parameters, "timeout", 30);

        let endpoint = format!("user/starred/{}/{}", owner, repo);
        GitHubApi::put(&endpoint, None, &token, api_url, timeout).await?;
        Ok(format!("Successfully starred {}/{}", owner, repo))
    }
}

// ========== Search repositories ==========
#[derive(Debug)]
pub struct GithubSearchRepos;

#[async_trait::async_trait]
impl Driver for GithubSearchRepos {
    fn name(&self) -> &str {
        "github_search_repos"
    }
    fn description(&self) -> &str {
        "Search GitHub repositories by query"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to find repositories"
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::Devops
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub personal access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description: "Search query (e.g., 'rust language:rust')".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust language:rust".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of results".to_string(),
                required: false,
                default: Some(Value::Number(10.into())),
                example: Some(Value::Number(5.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL".to_string(),
                required: false,
                default: Some(Value::String("https://api.github.com".to_string())),
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "github_search_repos", "parameters": { "token": "ghp_xxxxxxxx", "query": "rust language:rust", "limit": 5 } })
    }

    fn example_output(&self) -> String {
        r#"{"total_count": 12345, "items": [{"full_name": "rust-lang/rust", "description": "..."}]}"#.to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let token = get_param_string(parameters, "token")?;
        let query = get_param_string(parameters, "query")?;
        let limit = get_param_u64(parameters, "limit", 10);
        let api_url = parameters
            .get("api_url")
            .and_then(|v| v.as_str())
            .unwrap_or("https://api.github.com");
        let timeout = get_param_u64(parameters, "timeout", 30);

        let encoded_query = urlencoding::encode(&query);
        let endpoint = format!("search/repositories?q={}&per_page={}", encoded_query, limit);
        GitHubApi::get(&endpoint, &token, api_url, timeout).await
    }
}

// ========== Get user information ==========
#[derive(Debug)]
pub struct GithubGetUser;

#[async_trait::async_trait]
impl Driver for GithubGetUser {
    fn name(&self) -> &str {
        "github_get_user"
    }
    fn description(&self) -> &str {
        "Get GitHub user information"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to get profile info of a GitHub user"
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::Devops
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub personal access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "username".to_string(),
                param_type: "string".to_string(),
                description: "GitHub username".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("octocat".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL".to_string(),
                required: false,
                default: Some(Value::String("https://api.github.com".to_string())),
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "github_get_user", "parameters": { "token": "ghp_xxxxxxxx", "username": "octocat" } })
    }

    fn example_output(&self) -> String {
        r#"{"login": "octocat", "name": "The Octocat", "public_repos": 8}"#.to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let token = get_param_string(parameters, "token")?;
        let username = get_param_string(parameters, "username")?;
        let api_url = parameters
            .get("api_url")
            .and_then(|v| v.as_str())
            .unwrap_or("https://api.github.com");
        let timeout = get_param_u64(parameters, "timeout", 30);

        let endpoint = format!("users/{}", username);
        GitHubApi::get(&endpoint, &token, api_url, timeout).await
    }
}

// ========== List pull requests ==========
#[derive(Debug)]
pub struct GithubListPRs;

#[async_trait::async_trait]
impl Driver for GithubListPRs {
    fn name(&self) -> &str {
        "github_list_prs"
    }
    fn description(&self) -> &str {
        "List pull requests from a GitHub repository"
    }
    fn usage_hint(&self) -> &str {
        "Use this skill when the user needs to see open pull requests"
    }
    fn category(&self) -> DriverCategory {
        DriverCategory::Devops
    }

    fn parameters(&self) -> Vec<DriverParameter> {
        vec![
            DriverParameter {
                name: "token".to_string(),
                param_type: "string".to_string(),
                description: "GitHub personal access token".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("ghp_xxxxxxxx".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "owner".to_string(),
                param_type: "string".to_string(),
                description: "Repository owner".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust-lang".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "repo".to_string(),
                param_type: "string".to_string(),
                description: "Repository name".to_string(),
                required: true,
                default: None,
                example: Some(Value::String("rust".to_string())),
                enum_values: None,
            },
            DriverParameter {
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
            DriverParameter {
                name: "limit".to_string(),
                param_type: "integer".to_string(),
                description: "Maximum number of PRs to return".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(10.into())),
                enum_values: None,
            },
            DriverParameter {
                name: "api_url".to_string(),
                param_type: "string".to_string(),
                description: "GitHub API URL".to_string(),
                required: false,
                default: Some(Value::String("https://api.github.com".to_string())),
                example: Some(Value::String("https://api.github.com".to_string())),
                enum_values: None,
            },
            DriverParameter {
                name: "timeout".to_string(),
                param_type: "integer".to_string(),
                description: "Request timeout in seconds".to_string(),
                required: false,
                default: Some(Value::Number(30.into())),
                example: Some(Value::Number(60.into())),
                enum_values: None,
            },
        ]
    }

    fn example_call(&self) -> Value {
        json!({ "action": "github_list_prs", "parameters": { "token": "ghp_xxxxxxxx", "owner": "rust-lang", "repo": "rust", "state": "open", "limit": 10 } })
    }

    fn example_output(&self) -> String {
        r#"[{"number": 123, "title": "Add feature", "user": {"login": "contributor"}}]"#.to_string()
    }

    async fn execute(
        &self,
        parameters: &HashMap<String, Value>,
        callback: Option<&dyn DriverCallback>,
        context: Option<&DriverContext>,
    ) -> Result<String> {
        let token = get_param_string(parameters, "token")?;
        let owner = get_param_string(parameters, "owner")?;
        let repo = get_param_string(parameters, "repo")?;
        let state = parameters
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("open");
        let limit = get_param_u64(parameters, "limit", 30);
        let api_url = parameters
            .get("api_url")
            .and_then(|v| v.as_str())
            .unwrap_or("https://api.github.com");
        let timeout = get_param_u64(parameters, "timeout", 30);

        let endpoint = format!(
            "repos/{}/{}/pulls?state={}&per_page={}",
            owner, repo, state, limit
        );
        GitHubApi::get(&endpoint, &token, api_url, timeout).await
    }
}
